//! Chunk unloading systems for automatic chunk lifecycle management.
//!
//! This module provides configurable strategies for despawning chunks:
//!
//! - **Limit-based**: LRU eviction when chunk count exceeds a maximum
//! - **Distance-based** (requires `chunk_loader` feature): Unload chunks beyond
//!   a radius from all [`ChunkLoader`]s
//! - **Hybrid**: Both conditions must be met (enabled when both resources exist)
//!
//! # Enabling Unloading
//!
//! Unloading is opt-in via resources. Insert the appropriate resource(s) to enable:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use chunky_bevy::prelude::*;
//!
//! fn setup(mut commands: Commands) {
//!     // Limit-based only (always available)
//!     commands.insert_resource(ChunkUnloadLimit { max_chunks: 1000 });
//!
//!     // Distance-based only (requires chunk_loader feature)
//!     #[cfg(feature = "chunk_loader")]
//!     commands.insert_resource(ChunkUnloadByDistance);
//!
//!     // Hybrid: both must be true to unload
//!     #[cfg(feature = "chunk_loader")]
//!     {
//!         commands.insert_resource(ChunkUnloadByDistance);
//!         commands.insert_resource(ChunkUnloadLimit { max_chunks: 1000 });
//!     }
//! }
//! ```

use std::{collections::BinaryHeap, time::Instant};

use bevy::prelude::*;

use crate::{Chunk, ChunkManager, ChunkPos};

#[cfg(feature = "chunk_loader")]
use crate::chunk_loader::ChunkLoader;

pub struct ChunkUnloaderPlugin;

impl Plugin for ChunkUnloaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChunkUnloadEvent>();

        // Limit-based systems
        #[cfg(feature = "chunk_loader")]
        app.add_systems(
            PostUpdate,
            (
                init_chunk_last_access,
                update_chunk_last_access_by_limit,
                unload_chunks_by_limit,
            )
                .chain()
                .run_if(
                    resource_exists::<ChunkUnloadLimit>
                        .and(not(resource_exists::<ChunkUnloadByDistance>)),
                ),
        );

        #[cfg(not(feature = "chunk_loader"))]
        app.add_systems(
            PostUpdate,
            (
                init_chunk_last_access,
                update_chunk_last_access_by_limit,
                unload_chunks_by_limit,
            )
                .chain()
                .run_if(resource_exists::<ChunkUnloadLimit>),
        );

        // Distance-based and hybrid systems (require chunk_loader)
        #[cfg(feature = "chunk_loader")]
        {
            app.add_systems(
                PostUpdate,
                unload_chunks_by_distance.run_if(
                    resource_exists::<ChunkUnloadByDistance>
                        .and(not(resource_exists::<ChunkUnloadLimit>)),
                ),
            );

            app.add_systems(
                PostUpdate,
                (update_chunk_last_access_by_loader, unload_chunks_hybrid)
                    .chain()
                    .run_if(
                        resource_exists::<ChunkUnloadByDistance>
                            .and(resource_exists::<ChunkUnloadLimit>),
                    ),
            );

            #[cfg(feature = "reflect")]
            app.register_type::<ChunkUnloadByDistance>()
                .register_type::<ChunkUnloadRadius>();
        }

        #[cfg(feature = "reflect")]
        app.register_type::<ChunkUnloadLimit>()
            .register_type::<ChunkLastAccess>()
            .register_type::<ChunkPinned>()
            .register_type::<ChunkUnloadReason>();
    }
}

// ============================================================================
// Resources & Components
// ============================================================================

/// Marker resource that enables distance-based chunk unloading.
///
/// When present, chunks beyond the unload radius of all [`ChunkLoader`]s
/// will be despawned. The actual unload radius is defined per-loader via
/// [`ChunkUnloadRadius`].
///
/// Requires the `chunk_loader` feature.
#[derive(Resource, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct ChunkUnloadByDistance;

/// Defines the unload radius for a specific [`ChunkLoader`].
///
/// If absent on a loader, defaults to the loader's load radius (no buffer).
///
/// Requires the `chunk_loader` feature.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
///
/// fn spawn_player(mut commands: Commands) {
///     commands.spawn((
///         Transform::default(),
///         ChunkLoader(IVec3::new(3, 2, 3)),
///         // Unload chunks 2 beyond the load radius
///         ChunkUnloadRadius(IVec3::new(5, 4, 5)),
///     ));
/// }
/// ```
#[cfg(feature = "chunk_loader")]
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct ChunkUnloadRadius(pub IVec3);

/// When present, limits the total number of loaded chunks.
///
/// When the limit is exceeded, chunks are evicted based on [`ChunkLastAccess`],
/// starting with the least recently accessed.
///
/// If [`ChunkUnloadByDistance`] is also present (requires `chunk_loader` feature),
/// both conditions must be met: a chunk must be out of range AND the count must
/// exceed the limit.
#[derive(Resource, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct ChunkUnloadLimit {
    /// Maximum number of chunks to keep loaded.
    pub max_chunks: usize,
}

/// Tracks when a chunk was last "accessed" for LRU eviction.
///
/// Automatically updated based on the active unloading strategy:
/// - With `chunk_loader`: updated when within any [`ChunkLoader`]'s radius
/// - Without: updated for all loaded chunks each frame
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct ChunkLastAccess(pub Instant);

impl Default for ChunkLastAccess {
    fn default() -> Self {
        Self(Instant::now())
    }
}

/// Prevents a chunk from being automatically despawned.
///
/// Useful for pinning important chunks like spawn areas or quest locations.
#[derive(Component, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct ChunkPinned;

// ============================================================================
// Events
// ============================================================================

/// Fired when a chunk is about to be despawned by the unload system.
///
/// Users can observe this to save chunk data before removal.
#[derive(Message, Debug, Clone)]
pub struct ChunkUnloadEvent {
    pub entity: Entity,
    pub chunk_pos: IVec3,
    pub reason: ChunkUnloadReason,
}

/// Why a chunk is being unloaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum ChunkUnloadReason {
    /// Chunk exceeded distance from all loaders (requires `chunk_loader`)
    OutOfRange,
    /// Chunk evicted due to count limit (LRU)
    LimitExceeded,
    /// Both out of range and over limit (requires `chunk_loader`)
    Hybrid,
}

// ============================================================================
// Systems - Always Available
// ============================================================================

/// Updates [`ChunkLastAccess`] for chunks within any loader's radius (limit-only mode).
///
/// Without `chunk_loader` feature, this just ensures all chunks have the component.
/// With `chunk_loader` feature, this updates access time for chunks near loaders.
#[cfg(feature = "chunk_loader")]
fn update_chunk_last_access_by_limit(
    mut commands: Commands,
    loaders: Query<(&ChunkLoader, &GlobalTransform)>,
    mut chunks: Query<(Entity, &ChunkPos), With<Chunk>>,
    chunk_manager: Res<ChunkManager>,
) {
    let now = Instant::now();

    for (entity, chunk_pos) in chunks.iter_mut() {
        let in_range = loaders.iter().any(|(loader, transform)| {
            let loader_chunk = chunk_manager.get_chunk_pos(&transform.translation());
            is_within_radius(chunk_pos.0, loader_chunk, loader.0)
        });
        if in_range {
            commands.entity(entity).insert(ChunkLastAccess(now));
        }
    }
}

/// Updates [`ChunkLastAccess`] for all chunks (limit-only mode without chunk_loader).
///
/// Without loaders to determine "in range", we just ensure all chunks have the component.
/// Chunks keep their original spawn time, so oldest chunks get evicted first.
#[cfg(not(feature = "chunk_loader"))]
fn update_chunk_last_access_by_limit(
    mut commands: Commands,
    chunks: Query<(Entity, Option<&ChunkLastAccess>), With<Chunk>>,
) {
    for (entity, last_access) in chunks.iter() {
        if last_access.is_none() {
            commands.entity(entity).insert(ChunkLastAccess::default());
        }
    }
}

/// Limit-based LRU unloading only.
fn unload_chunks_by_limit(
    mut commands: Commands,
    mut unload_events: MessageWriter<ChunkUnloadEvent>,
    chunks: Query<(Entity, &ChunkPos, &ChunkLastAccess), (With<Chunk>, Without<ChunkPinned>)>,
    limit: Res<ChunkUnloadLimit>,
) {
    if chunks.iter().count() <= limit.max_chunks {
        return;
    }
    // Collect and sort by last access (oldest first)
    let mut candidates = BinaryHeap::new();
    for (entity, chunk_pos, last_access) in chunks.iter() {
        candidates.push(Candidate {
            time: last_access.0,
            entity,
            pos: chunk_pos.0,
        });
    }

    while candidates.len() > limit.max_chunks {
        let Some(Candidate { entity, pos, .. }) = candidates.pop() else {
            panic!();
        };
        unload_events.write(ChunkUnloadEvent {
            entity,
            chunk_pos: pos,
            reason: ChunkUnloadReason::LimitExceeded,
        });
        commands.entity(entity).despawn();
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Candidate {
    pub time: Instant,
    pub entity: Entity,
    pub pos: IVec3,
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use core::cmp::Ordering;
        //we want the oldest first
        match self.time.partial_cmp(&other.time) {
            Some(Ordering::Equal) | None => self.entity.partial_cmp(&other.entity),
            Some(Ordering::Less) => Some(Ordering::Greater),
            Some(Ordering::Greater) => Some(Ordering::Less),
        }
    }
}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use core::cmp::Ordering;
        //we want the oldest first
        match self.time.cmp(&other.time) {
            Ordering::Equal => self.entity.cmp(&other.entity),
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
        }
    }
}

fn init_chunk_last_access(
    mut commands: Commands,
    chunks: Query<Entity, (With<Chunk>, Without<ChunkLastAccess>)>,
) {
    for entity in chunks.iter() {
        commands
            .entity(entity)
            .insert(ChunkLastAccess(Instant::now()));
    }
}

// ============================================================================
// Systems - Require chunk_loader
// ============================================================================

/// Updates [`ChunkLastAccess`] for chunks within any loader's radius.
#[cfg(feature = "chunk_loader")]
fn update_chunk_last_access_by_loader(
    loaders: Query<(&ChunkLoader, &GlobalTransform)>,
    mut chunks: Query<(&ChunkPos, &mut ChunkLastAccess), With<Chunk>>,
    chunk_manager: Res<ChunkManager>,
) {
    let now = Instant::now();

    for (chunk_pos, mut last_access) in chunks.iter_mut() {
        let in_range = loaders.iter().any(|(loader, transform)| {
            let loader_chunk = chunk_manager.get_chunk_pos(&transform.translation());
            is_within_radius(chunk_pos.0, loader_chunk, loader.0)
        });
        if in_range {
            last_access.0 = now;
        }
    }
}

/// Distance-based unloading only.
#[cfg(feature = "chunk_loader")]
fn unload_chunks_by_distance(
    mut commands: Commands,
    mut unload_events: MessageWriter<ChunkUnloadEvent>,
    loaders: Query<(&ChunkLoader, Option<&ChunkUnloadRadius>, &GlobalTransform)>,
    chunks: Query<(Entity, &ChunkPos), (With<Chunk>, Without<ChunkPinned>)>,
    chunk_manager: Res<ChunkManager>,
) {
    for (entity, chunk_pos) in chunks.iter() {
        if !is_in_any_unload_radius(chunk_pos.0, &loaders, &chunk_manager) {
            unload_events.write(ChunkUnloadEvent {
                entity,
                chunk_pos: chunk_pos.0,
                reason: ChunkUnloadReason::OutOfRange,
            });
            commands.entity(entity).despawn();
        }
    }
}

/// Hybrid unloading: both out of range AND over limit.
#[cfg(feature = "chunk_loader")]
fn unload_chunks_hybrid(
    mut commands: Commands,
    mut unload_events: MessageWriter<ChunkUnloadEvent>,
    loaders: Query<(&ChunkLoader, Option<&ChunkUnloadRadius>, &GlobalTransform)>,
    chunks: Query<
        (Entity, &ChunkPos, Option<&ChunkLastAccess>),
        (With<Chunk>, Without<ChunkPinned>),
    >,
    chunk_manager: Res<ChunkManager>,
    limit: Res<ChunkUnloadLimit>,
) {
    let chunk_count = chunks.iter().count();

    if chunk_count <= limit.max_chunks {
        return;
    }

    let to_remove = chunk_count - limit.max_chunks;

    // Only consider chunks that are out of range
    let mut candidates: Vec<_> = chunks
        .iter()
        .filter(|(_, chunk_pos, _)| !is_in_any_unload_radius(chunk_pos.0, &loaders, &chunk_manager))
        .map(|(e, pos, access)| {
            let time = access.map(|a| a.0).unwrap_or(Instant::now());
            (e, pos.0, time)
        })
        .collect();

    // Sort by oldest first
    candidates.sort_by_key(|(_, _, time)| *time);

    for (entity, chunk_pos, _) in candidates.into_iter().take(to_remove) {
        unload_events.write(ChunkUnloadEvent {
            entity,
            chunk_pos,
            reason: ChunkUnloadReason::Hybrid,
        });
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Checks if a chunk is within any loader's unload radius.
#[cfg(feature = "chunk_loader")]
fn is_in_any_unload_radius(
    chunk_pos: IVec3,
    loaders: &Query<(&ChunkLoader, Option<&ChunkUnloadRadius>, &GlobalTransform)>,
    chunk_manager: &ChunkManager,
) -> bool {
    loaders.iter().any(|(loader, unload_radius, transform)| {
        let loader_chunk = chunk_manager.get_chunk_pos(&transform.translation());
        let radius = unload_radius.map(|r| r.0).unwrap_or(loader.0);
        is_within_radius(chunk_pos, loader_chunk, radius)
    })
}

/// Checks if `pos` is within `radius` of `center` (per-axis).
#[cfg(feature = "chunk_loader")]
fn is_within_radius(pos: IVec3, center: IVec3, radius: IVec3) -> bool {
    let diff = (pos - center).abs();
    diff.x <= radius.x && diff.y <= radius.y && diff.z <= radius.z
}

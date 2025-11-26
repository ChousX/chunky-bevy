//! A simple and efficient chunk management system for Bevy.
//!
//! # Quick Start
//!
//! ```no_run
//! use bevy::prelude::*;
//! use chunky_bevy::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(ChunkyPlugin::default())
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Spawn a chunk loader that generates chunks around it
//!     commands.spawn((
//!         Transform::default(),
//!         ChunkLoader(IVec3::new(2, 1, 2)), // Load 5x3x5 chunks
//!     ));
//! }
//! ```
//!
//! # Features
//!
//! - `chunk_visualizer` (default) - Enables debug visualization of chunk boundaries
//! - `chunk_loader` (default) - Enables automatic chunk loading around ChunkLoader entities
//! - `chunk_info` - Logs chunk spawn/despawn events

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use std::collections::HashMap;

/// Re-exports of commonly used types
pub mod prelude {
    #[cfg(feature = "chunk_visualizer")]
    pub use crate::ChunkBoundryVisualizer;
    #[cfg(feature = "chunk_loader")]
    pub use crate::ChunkLoader;
    pub use crate::{Chunk, ChunkManager, ChunkPos, ChunkyPlugin};
}

/// The main plugin for chunk management.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::ChunkyPlugin;
///
/// App::new()
///     .add_plugins(ChunkyPlugin::default()) // 10x10x10 chunks
///     .run();
/// ```
pub struct ChunkyPlugin {
    chunk_size: Vec3,
}

impl Plugin for ChunkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkManager::new(self.chunk_size));
        #[cfg(feature = "chunk_visualizer")]
        app.init_state::<ChunkBoundryVisualizer>().add_systems(
            Update,
            chunk_boundry_visualizer.run_if(in_state(ChunkBoundryVisualizer::On)),
        );
        #[cfg(feature = "chunk_loader")]
        app.add_systems(Update, chunk_loader);
    }
}

impl ChunkyPlugin {
    /// Standard 3D chunk configuration with 10x10x10 sized chunks
    pub const THREE_DIMETION: Self = Self {
        chunk_size: vec3(10.0, 10.0, 10.0),
    };
}

impl Default for ChunkyPlugin {
    fn default() -> Self {
        Self::THREE_DIMETION
    }
}

/// Utility functions for spawning chunks in bulk
pub mod helpers {
    use crate::{Chunk, ChunkPos};
    use bevy::prelude::*;

    /// Spawns chunks in a rectangular region defined by two chunk positions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use chunky::helpers::*;
    ///
    /// fn setup(mut commands: Commands) {
    ///     // Spawn a 6x6x6 cube of chunks from (0,0,0) to (5,5,5)
    ///     spawn_chunks_rect(&mut commands, IVec3::ZERO, IVec3::splat(5));
    /// }
    /// ```
    pub fn spawn_chunks_rect(commands: &mut Commands, chunk_pos_0: IVec3, chunk_pos_1: IVec3) {
        let (x_small, x_big) = if chunk_pos_0.x > chunk_pos_1.x {
            (chunk_pos_1.x, chunk_pos_0.x)
        } else {
            (chunk_pos_0.x, chunk_pos_1.x)
        };
        let (y_small, y_big) = if chunk_pos_0.y > chunk_pos_1.y {
            (chunk_pos_1.y, chunk_pos_0.y)
        } else {
            (chunk_pos_0.y, chunk_pos_1.y)
        };
        let (z_small, z_big) = if chunk_pos_0.z > chunk_pos_1.z {
            (chunk_pos_1.z, chunk_pos_0.z)
        } else {
            (chunk_pos_0.z, chunk_pos_1.z)
        };
        for x in x_small..=x_big {
            for y in y_small..=y_big {
                for z in z_small..=z_big {
                    let chunk_pos = ivec3(x, y, z);
                    commands.spawn((Chunk, ChunkPos(chunk_pos)));
                }
            }
        }
    }

    /// Spawns chunks in a rectangular region defined by two world positions.
    ///
    /// The world positions are converted to chunk positions before spawning.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use chunky_bevy::helpers::*;
    ///
    /// fn setup(mut commands: Commands) {
    ///     // Spawn chunks covering the world space from (0,0,0) to (100,50,100)
    ///     spawn_chunks_rect_from_world_pos(
    ///         &mut commands,
    ///         Vec3::ZERO,
    ///         Vec3::new(100.0, 50.0, 100.0)
    ///     );
    /// }
    /// ```
    pub fn spawn_chunks_rect_from_world_pos(
        commands: &mut Commands,
        chunk_pos_0: Vec3,
        chunk_pos_1: Vec3,
    ) {
        let chunk_pos_0 = chunk_pos_0.floor().as_ivec3();
        let chunk_pos_1 = chunk_pos_1.floor().as_ivec3();
        spawn_chunks_rect(commands, chunk_pos_0, chunk_pos_1);
    }
}

/// Marks an entity as a chunk.
///
/// This component automatically:
/// - Registers the chunk with the [`ChunkManager`] when added
/// - Unregisters the chunk when removed
/// - Requires [`ChunkPos`] and [`Visibility`] components
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
///
/// fn spawn_chunk(mut commands: Commands) {
///     commands.spawn((
///         Chunk,
///         ChunkPos(IVec3::new(0, 0, 0)),
///     ));
/// }
/// ```
#[derive(Component)]
#[require(ChunkPos, Visibility)]
#[component(
    immutable,
    on_add= on_add_chunk,
    on_remove = on_remove_chunk
)]
pub struct Chunk;

/// Adds Chunk to ChunkManager
fn on_add_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    let mut chunk_manager = world.get_resource_mut::<ChunkManager>().unwrap();
    if chunk_manager.is_loaded(&chunk_pos) {
        warn!(
            "New chunk at pos:{} was not spawned there was already a chunk there",
            chunk_pos
        );
        return;
    }

    chunk_manager.insert(chunk_pos, entity);

    #[cfg(feature = "chunk_info")]
    info!("[ChunkInfo]ChunkPos: {chunk_pos:?}");
}

/// Removes Chunk from ChunkManager
fn on_remove_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    world
        .get_resource_mut::<ChunkManager>()
        .unwrap()
        .remove(&chunk_pos);
}

/// The position of a chunk in chunk-space coordinates.
///
/// When added to an entity, automatically updates the entity's [`Transform`]
/// to match the chunk's world position.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
///
/// fn spawn_chunk(mut commands: Commands) {
///     // Spawns a chunk at chunk position (5, 0, 3)
///     // With default 10x10x10 chunks, this will be at world position (50, 0, 30)
///     commands.spawn((
///         Chunk,
///         ChunkPos(IVec3::new(5, 0, 3)),
///     ));
/// }
/// ```
#[derive(Component, Default, Deref, DerefMut)]
#[require(Transform)]
#[component(
    immutable,
    on_add= on_add_chunk_pos,
)]
pub struct ChunkPos(pub IVec3);

/// Updates Transform to match ChunkPos
fn on_add_chunk_pos(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap();
    let chunk_size = world.get_resource::<ChunkManager>().unwrap().chunk_size;
    let translation = chunk_pos.as_vec3() * chunk_size;
    world.get_mut::<Transform>(entity).unwrap().translation = translation;
}

/// Resource for managing all chunks in the world.
///
/// Provides methods to query chunks by position and convert between
/// world positions and chunk positions.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
///
/// fn check_chunk(
///     chunk_manager: Res<ChunkManager>,
///     player_pos: Vec3,
/// ) {
///     // Get the chunk position the player is in
///     let chunk_pos = chunk_manager.get_chunk_pos(&player_pos);
///     
///     // Check if that chunk is loaded
///     if chunk_manager.is_loaded(&chunk_pos) {
///         println!("Player is in a loaded chunk!");
///     }
/// }
/// ```
#[derive(Resource, Default)]
pub struct ChunkManager {
    chunk_size: Vec3,
    chunks: HashMap<IVec3, Entity>,
}

impl ChunkManager {
    /// Creates a new chunk manager with the specified chunk size
    pub fn new(chunk_size: Vec3) -> Self {
        Self {
            chunk_size,
            chunks: default(),
        }
    }

    /// Returns the size of chunks in world units
    pub fn get_size(&self) -> Vec3 {
        self.chunk_size
    }

    /// Inserts a new chunk into the manager.
    ///
    /// Returns the previous chunk entity if one already existed at this position.
    ///
    /// Note: This is called automatically when a [`Chunk`] component is added.
    pub fn insert(&mut self, pos: IVec3, id: Entity) -> Option<Entity> {
        self.chunks.insert(pos, id)
    }

    /// Removes a chunk from the manager.
    ///
    /// Returns the chunk's entity if it existed.
    ///
    /// Note: This is called automatically when a [`Chunk`] component is removed.
    pub fn remove(&mut self, pos: &IVec3) -> Option<Entity> {
        self.chunks.remove(pos)
    }

    /// Converts world coordinates into chunk position
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bevy::prelude::*;
    /// use chunky_bevy::prelude::*;
    ///
    /// fn example(chunk_manager: Res<ChunkManager>) {
    ///     let world_pos = Vec3::new(15.0, 5.0, 23.0);
    ///     let chunk_pos = chunk_manager.get_chunk_pos(&world_pos);
    ///     // With default 10x10x10 chunks, this returns IVec3(1, 0, 2)
    /// }
    /// ```
    pub fn get_chunk_pos(&self, pos: &Vec3) -> IVec3 {
        (*pos / self.chunk_size).floor().as_ivec3()
    }

    /// Gets the chunk entity at the specified chunk position if it exists
    pub fn get_chunk(&self, chunk_pos: &IVec3) -> Option<Entity> {
        self.chunks.get(chunk_pos).copied()
    }

    /// Gets the chunk entity at the specified world position if it exists
    pub fn get_chunk_form_pos(&self, pos: &Vec3) -> Option<Entity> {
        self.get_chunk(&self.get_chunk_pos(pos))
    }

    /// Checks if a chunk is loaded at the specified chunk position
    pub fn is_loaded(&self, chunk_pos: &IVec3) -> bool {
        self.chunks.contains_key(chunk_pos)
    }
}

/// Automatically loads chunks around the entity.
///
/// The `IVec3` defines the loading radius in each direction from the chunk
/// the entity is currently in.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
///
/// fn spawn_player(mut commands: Commands) {
///     commands.spawn((
///         Transform::default(),
///         // Load only the chunk the player is in
///         ChunkLoader(IVec3::ZERO),
///     ));
///     
///     commands.spawn((
///         Transform::default(),
///         // Load a 3x3x3 cube of chunks (1 in each direction)
///         ChunkLoader(IVec3::ONE),
///     ));
///     
///     commands.spawn((
///         Transform::default(),
///         // Load a 11x1x11 flat area (5 chunks in each horizontal direction)
///         ChunkLoader(IVec3::new(5, 0, 5)),
///     ));
/// }
/// ```
#[derive(Component, Default, Debug)]
pub struct ChunkLoader(pub IVec3);

/// Load Chunks Around ChunkLoader
fn chunk_loader(
    chunks: Query<(&ChunkLoader, &GlobalTransform)>,
    chunk_manager: Res<ChunkManager>,
    mut commands: Commands,
) {
    for (ChunkLoader(loading_radius), g_transform) in chunks.iter() {
        let translation = g_transform.translation();
        let in_chunk = chunk_manager.get_chunk_pos(&translation);
        for x in -loading_radius.x..=loading_radius.x {
            for y in -loading_radius.y..=loading_radius.y {
                for z in -loading_radius.z..=loading_radius.z {
                    let target_chunk = in_chunk + ivec3(x, y, z);
                    if !chunk_manager.is_loaded(&target_chunk) {
                        commands.spawn((Chunk, ChunkPos(target_chunk)));
                    }
                }
            }
        }
    }
}

/// State for controlling chunk boundary visualization
#[cfg(feature = "chunk_visualizer")]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ChunkBoundryVisualizer {
    /// Chunk boundaries are visible
    On,
    /// Chunk boundaries are hidden (default)
    #[default]
    Off,
}

/// Shows all existing chunk boundaries using gizmos
#[cfg(feature = "chunk_visualizer")]
fn chunk_boundry_visualizer(
    chunk_manager: Res<ChunkManager>,
    chunks: Query<&ChunkPos>,
    mut gizmos: Gizmos,
) {
    let chunk_size = chunk_manager.get_size();

    for ChunkPos(chunk_pos) in chunks.iter() {
        let origin = chunk_pos.as_vec3() * chunk_size;

        // 8 corners of the box
        let p000 = origin;
        let p100 = origin + Vec3::new(chunk_size.x, 0.0, 0.0);
        let p010 = origin + Vec3::new(0.0, chunk_size.y, 0.0);
        let p110 = origin + Vec3::new(chunk_size.x, chunk_size.y, 0.0);

        let p001 = origin + Vec3::new(0.0, 0.0, chunk_size.z);
        let p101 = origin + Vec3::new(chunk_size.x, 0.0, chunk_size.z);
        let p011 = origin + Vec3::new(0.0, chunk_size.y, chunk_size.z);
        let p111 = origin + Vec3::new(chunk_size.x, chunk_size.y, chunk_size.z);

        let color = bevy::color::palettes::tailwind::GREEN_500;

        // bottom rectangle
        gizmos.line(p000, p100, color);
        gizmos.line(p100, p110, color);
        gizmos.line(p110, p010, color);
        gizmos.line(p010, p000, color);

        // top rectangle
        gizmos.line(p001, p101, color);
        gizmos.line(p101, p111, color);
        gizmos.line(p111, p011, color);
        gizmos.line(p011, p001, color);

        // vertical edges
        gizmos.line(p000, p001, color);
        gizmos.line(p100, p101, color);
        gizmos.line(p110, p111, color);
        gizmos.line(p010, p011, color);
    }
}

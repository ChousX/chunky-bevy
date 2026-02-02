//! Chunk persistence module.
//!
//! Register serializable components to be saved/loaded with chunks.
//!
//! # Example
//!
//! ```no_run
//! use bevy::prelude::*;
//! use chunky_bevy::prelude::*;
//! use chunky_bevy::saving::prelude::*;
//!
//! App::new()
//!     .add_plugins(ChunkSavingPlugin::new("saves/world"))
//!     .register_chunk_data::<MyVoxelData>()
//!     .run();
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{any::TypeId, collections::HashMap, fs, path::PathBuf};

use crate::{
    ChunkySet,
    core::{Chunk, ChunkManager, ChunkPosition},
};

#[cfg(feature = "chunk_unloader")]
use crate::chunk_unloader::ChunkUnloadLimit;

/// Sent when a chunk is spawned and has no saved data on disk.
///
/// Listen to this event to generate terrain, voxels, or other initial chunk data.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::prelude::*;
/// use chunky_bevy::saving::prelude::*;
///
/// fn generate_terrain(
///     mut commands: Commands,
///     mut events: EventReader<ChunkNeedsGeneration>,
/// ) {
///     for event in events.read() {
///         let voxels = generate_voxels_for(event.pos);
///         commands.entity(event.entity).insert(voxels);
///     }
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct ChunkNeedsGeneration {
    pub entity: Entity,
    pub pos: IVec3,
}

pub struct ChunkSavingPlugin {
    base_path: PathBuf,
    style: SaveStyle,
    auto_save: bool,
    auto_load: bool,
}

/// How chunks are organized into files.
#[derive(Clone, Debug)]
pub enum SaveStyle {
    /// One file per chunk.
    PerChunk,
    /// Multiple chunks grouped into a single file.
    SuperChunk { size: UVec3 },
}

impl ChunkSavingPlugin {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            style: SaveStyle::PerChunk,
            auto_save: false,
            auto_load: false,
        }
    }

    pub fn with_style(mut self, style: SaveStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable auto-save when chunks are unloaded (requires `chunk_unloader` feature).
    pub fn with_auto_save(mut self) -> Self {
        self.auto_save = true;
        self
    }

    /// Enable auto-load when chunks are spawned.
    pub fn with_auto_load(mut self) -> Self {
        self.auto_load = true;
        self
    }
}

impl Plugin for ChunkSavingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkSaveConfig {
            base_path: self.base_path.clone(),
            style: self.style.clone(),
            auto_save: self.auto_save,
            auto_load: self.auto_load,
        })
        .init_resource::<ChunkDataRegistry>();

        // Auto-save before chunk unload
        #[cfg(feature = "chunk_unloader")]
        if self.auto_save {
            use crate::ChunkySet;

            app.add_systems(
                Update,
                (mark_chunks_for_save, auto_save_before_unload)
                    .chain()
                    .in_set(ChunkySet::Save),
            );
        }

        // Auto-load on chunk spawn
        if self.auto_load {
            app.add_systems(Update, auto_load_on_spawn.in_set(ChunkySet::Load));
        }
    }
}

#[derive(Resource)]
pub struct ChunkSaveConfig {
    pub base_path: PathBuf,
    pub style: SaveStyle,
    #[allow(dead_code)]
    pub auto_save: bool,
    #[allow(dead_code)]
    pub auto_load: bool,
}

impl ChunkSaveConfig {
    /// Get the file path for a chunk position.
    pub fn chunk_path(&self, pos: IVec3) -> PathBuf {
        match &self.style {
            SaveStyle::PerChunk => self
                .base_path
                .join(format!("chunk_{}_{}_{}.chunk", pos.x, pos.y, pos.z)),
            SaveStyle::SuperChunk { size } => {
                let super_pos = Self::super_chunk_pos(pos, *size);
                self.base_path.join(format!(
                    "super_{}_{}_{}.chunks",
                    super_pos.x, super_pos.y, super_pos.z
                ))
            }
        }
    }

    /// Get the super-chunk position that contains a chunk position.
    fn super_chunk_pos(chunk_pos: IVec3, size: UVec3) -> IVec3 {
        let size = size.as_ivec3();
        IVec3::new(
            chunk_pos.x.div_euclid(size.x),
            chunk_pos.y.div_euclid(size.y),
            chunk_pos.z.div_euclid(size.z),
        )
    }
}

#[derive(Resource, Default)]
pub struct ChunkDataRegistry {
    serializers: HashMap<TypeId, ComponentSerializer>,
}

struct ComponentSerializer {
    type_name: String,
    extract: Box<dyn Fn(&World, Entity) -> Option<Vec<u8>> + Send + Sync>,
    insert: Box<dyn Fn(&mut Commands, Entity, &[u8]) + Send + Sync>,
}

pub trait RegisterChunkData {
    fn register_chunk_data<T: Component + Serialize + DeserializeOwned>(&mut self) -> &mut Self;
}

impl RegisterChunkData for App {
    fn register_chunk_data<T: Component + Serialize + DeserializeOwned>(&mut self) -> &mut Self {
        let serializer = ComponentSerializer {
            type_name: std::any::type_name::<T>().to_string(),
            extract: Box::new(|world, entity| {
                world
                    .get::<T>(entity)
                    .and_then(|c| postcard::to_allocvec(c).ok())
            }),
            insert: Box::new(|commands, entity, bytes| {
                if let Ok(component) = postcard::from_bytes::<T>(bytes) {
                    commands.entity(entity).insert(component);
                }
            }),
        };

        self.world_mut()
            .resource_mut::<ChunkDataRegistry>()
            .serializers
            .insert(TypeId::of::<T>(), serializer);

        self
    }
}

#[derive(Serialize, Deserialize)]
struct ChunkFile {
    pos: IVec3,
    components: Vec<(String, Vec<u8>)>,
}

#[derive(Serialize, Deserialize, Default)]
struct SuperChunkFile {
    chunks: HashMap<IVec3, Vec<(String, Vec<u8>)>>,
}

#[allow(dead_code)]
impl ChunkDataRegistry {
    /// Save a chunk entity to disk.
    pub fn save(
        &self,
        world: &World,
        entity: Entity,
        config: &ChunkSaveConfig,
    ) -> Result<(), SaveError> {
        let pos = world
            .get::<ChunkPosition>(entity)
            .ok_or(SaveError::NotAChunk)?
            .0;

        let components: Vec<_> = self
            .serializers
            .values()
            .filter_map(|s| (s.extract)(world, entity).map(|data| (s.type_name.clone(), data)))
            .collect();

        match &config.style {
            SaveStyle::PerChunk => {
                let file = ChunkFile { pos, components };
                let bytes = postcard::to_allocvec(&file)
                    .map_err(|e| SaveError::Serialize(e.to_string()))?;

                let path = config.chunk_path(pos);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
                }
                fs::write(&path, bytes).map_err(|e| SaveError::Io(e.to_string()))?;
            }
            SaveStyle::SuperChunk { size: _ } => {
                let path = config.chunk_path(pos);

                // Load existing super-chunk or create new
                let mut super_file: SuperChunkFile = if path.exists() {
                    let bytes = fs::read(&path).map_err(|e| SaveError::Io(e.to_string()))?;
                    postcard::from_bytes(&bytes).unwrap_or_default()
                } else {
                    SuperChunkFile::default()
                };

                super_file.chunks.insert(pos, components);

                let bytes = postcard::to_allocvec(&super_file)
                    .map_err(|e| SaveError::Serialize(e.to_string()))?;

                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
                }
                fs::write(&path, bytes).map_err(|e| SaveError::Io(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Save multiple chunks at once. More efficient for SuperChunk style
    /// as it batches writes to the same file.
    pub fn save_batch(
        &self,
        world: &World,
        entities: &[Entity],
        config: &ChunkSaveConfig,
    ) -> Result<(), SaveError> {
        match &config.style {
            SaveStyle::PerChunk => {
                for &entity in entities {
                    self.save(world, entity, config)?;
                }
            }
            SaveStyle::SuperChunk { size } => {
                // Group entities by their super-chunk
                let mut by_super: HashMap<IVec3, Vec<(IVec3, Vec<(String, Vec<u8>)>)>> =
                    HashMap::new();

                for &entity in entities {
                    let pos = world
                        .get::<ChunkPosition>(entity)
                        .ok_or(SaveError::NotAChunk)?
                        .0;

                    let components: Vec<_> = self
                        .serializers
                        .values()
                        .filter_map(|s| {
                            (s.extract)(world, entity).map(|data| (s.type_name.clone(), data))
                        })
                        .collect();

                    let super_pos = ChunkSaveConfig::super_chunk_pos(pos, *size);
                    by_super
                        .entry(super_pos)
                        .or_default()
                        .push((pos, components));
                }

                // Write each super-chunk file once
                for (super_pos, chunks) in by_super {
                    let path = config.base_path.join(format!(
                        "super_{}_{}_{}.chunks",
                        super_pos.x, super_pos.y, super_pos.z
                    ));

                    // Load existing or create new
                    let mut super_file: SuperChunkFile = if path.exists() {
                        let bytes = fs::read(&path).map_err(|e| SaveError::Io(e.to_string()))?;
                        postcard::from_bytes(&bytes).unwrap_or_default()
                    } else {
                        SuperChunkFile::default()
                    };

                    // Insert all chunks for this super-chunk
                    for (pos, components) in chunks {
                        super_file.chunks.insert(pos, components);
                    }

                    let bytes = postcard::to_allocvec(&super_file)
                        .map_err(|e| SaveError::Serialize(e.to_string()))?;

                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
                    }
                    fs::write(&path, bytes).map_err(|e| SaveError::Io(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// Load chunk data into an entity.
    pub fn load(
        &self,
        commands: &mut Commands,
        entity: Entity,
        config: &ChunkSaveConfig,
        pos: IVec3,
    ) -> Result<(), SaveError> {
        let path = config.chunk_path(pos);
        let bytes = fs::read(&path).map_err(|e| SaveError::Io(e.to_string()))?;

        let components = match &config.style {
            SaveStyle::PerChunk => {
                let file: ChunkFile = postcard::from_bytes(&bytes)
                    .map_err(|e| SaveError::Deserialize(e.to_string()))?;
                file.components
            }
            SaveStyle::SuperChunk { .. } => {
                let file: SuperChunkFile = postcard::from_bytes(&bytes)
                    .map_err(|e| SaveError::Deserialize(e.to_string()))?;
                file.chunks
                    .get(&pos)
                    .cloned()
                    .ok_or(SaveError::ChunkNotFound(pos))?
            }
        };

        let by_name: HashMap<&str, &ComponentSerializer> = self
            .serializers
            .values()
            .map(|s| (s.type_name.as_str(), s))
            .collect();

        for (type_name, data) in &components {
            if let Some(s) = by_name.get(type_name.as_str()) {
                (s.insert)(commands, entity, data);
            }
        }

        Ok(())
    }

    /// Load all chunks from a super-chunk file. Returns the positions that were loaded.
    /// For PerChunk style, loads a single chunk at the given position.
    pub fn load_batch(
        &self,
        commands: &mut Commands,
        config: &ChunkSaveConfig,
        pos: IVec3,
    ) -> Result<Vec<(IVec3, Entity)>, SaveError> {
        let path = config.chunk_path(pos);
        let bytes = fs::read(&path).map_err(|e| SaveError::Io(e.to_string()))?;

        let by_name: HashMap<&str, &ComponentSerializer> = self
            .serializers
            .values()
            .map(|s| (s.type_name.as_str(), s))
            .collect();

        let mut loaded = Vec::new();

        match &config.style {
            SaveStyle::PerChunk => {
                let file: ChunkFile = postcard::from_bytes(&bytes)
                    .map_err(|e| SaveError::Deserialize(e.to_string()))?;

                let entity = commands
                    .spawn((Chunk, ChunkPosition(file.pos), ChunkLoadedFromDisk))
                    .id();

                for (type_name, data) in &file.components {
                    if let Some(s) = by_name.get(type_name.as_str()) {
                        (s.insert)(commands, entity, data);
                    }
                }

                loaded.push((file.pos, entity));
            }
            SaveStyle::SuperChunk { .. } => {
                let file: SuperChunkFile = postcard::from_bytes(&bytes)
                    .map_err(|e| SaveError::Deserialize(e.to_string()))?;

                for (chunk_pos, components) in file.chunks {
                    let entity = commands
                        .spawn((Chunk, ChunkPosition(chunk_pos), ChunkLoadedFromDisk))
                        .id();

                    for (type_name, data) in &components {
                        if let Some(s) = by_name.get(type_name.as_str()) {
                            (s.insert)(commands, entity, data);
                        }
                    }

                    loaded.push((chunk_pos, entity));
                }
            }
        }

        Ok(loaded)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SaveError {
    NotAChunk,
    ChunkNotFound(IVec3),
    Io(String),
    Serialize(String),
    Deserialize(String),
}

// ============================================================================
// Auto-save/load systems
// ============================================================================

/// Marker component to prevent auto-loading a chunk that was just spawned by load_batch.
#[derive(Component)]
struct ChunkLoadedFromDisk;

/// Marker for chunks pending save before unload.
#[cfg(feature = "chunk_unloader")]
#[derive(Component)]
pub struct ChunkPendingSave;

/// Mark chunks for saving when they're about to be unloaded.
/// This runs before the unloader systems identify chunks to remove.
#[cfg(feature = "chunk_unloader")]
fn mark_chunks_for_save(
    mut commands: Commands,
    chunks: Query<
        Entity,
        (
            With<Chunk>,
            Without<ChunkPendingSave>,
            Without<ChunkLoadedFromDisk>,
        ),
    >,
) {
    // Mark all chunks - the actual save happens in process_pending_saves
    // which runs right before unload
    for entity in chunks.iter() {
        commands.entity(entity).insert(ChunkPendingSave);
    }
}

/// Save chunks that are marked and about to be unloaded.
#[cfg(feature = "chunk_unloader")]
fn auto_save_before_unload(
    world: &World,
    chunks_to_unload: Query<(Entity, &ChunkPosition), (With<Chunk>, With<ChunkPendingSave>)>,
    registry: Res<ChunkDataRegistry>,
    config: Res<ChunkSaveConfig>,
    chunk_manager: Res<ChunkManager>,
    #[cfg(feature = "chunk_loader")] loaders: Query<(
        &crate::chunk_loader::ChunkLoader,
        Option<&crate::chunk_unloader::ChunkUnloadRadius>,
        &GlobalTransform,
    )>,
    limit: Option<Res<ChunkUnloadLimit>>,
) {
    // Determine which chunks will be unloaded this frame
    let chunk_count = chunks_to_unload.iter().count();

    for (entity, chunk_pos) in chunks_to_unload.iter() {
        let mut will_unload = false;

        // Check distance-based unload
        #[cfg(feature = "chunk_loader")]
        {
            let in_range = loaders.iter().any(|(loader, unload_radius, transform)| {
                let loader_chunk = chunk_manager.get_chunk_pos(&transform.translation());
                let radius = unload_radius.map(|r| r.0).unwrap_or(loader.0);
                let diff = (chunk_pos.0 - loader_chunk).abs();
                diff.x <= radius.x && diff.y <= radius.y && diff.z <= radius.z
            });
            if !in_range {
                will_unload = true;
            }
        }

        // Check limit-based unload
        if let Some(ref limit) = limit
            && chunk_count > limit.max_chunks
        {
            will_unload = true;
        }

        if will_unload && let Err(e) = registry.save(world, entity, &config) {
            error!("Failed to auto-save chunk {:?}: {:?}", chunk_pos.0, e);
        }
    }
}

/// Auto-load chunk data when new chunks are spawned.
fn _auto_load_on_spawn(
    mut commands: Commands,
    new_chunks: Query<(Entity, &ChunkPosition), (Added<Chunk>, Without<ChunkLoadedFromDisk>)>,
    registry: Res<ChunkDataRegistry>,
    config: Res<ChunkSaveConfig>,
) {
    for (entity, chunk_pos) in new_chunks.iter() {
        let path = config.chunk_path(chunk_pos.0);
        if path.exists()
            && let Err(e) = registry.load(&mut commands, entity, &config, chunk_pos.0)
        {
            error!("Failed to auto-load chunk {:?}: {:?}", chunk_pos.0, e);
        }
    }
}

/// Auto-load chunk data when new chunks are spawned.
fn auto_load_on_spawn(
    mut commands: Commands,
    new_chunks: Query<(Entity, &ChunkPosition), (Added<Chunk>, Without<ChunkLoadedFromDisk>)>,
    registry: Res<ChunkDataRegistry>,
    config: Res<ChunkSaveConfig>,
) {
    for (entity, chunk_pos) in new_chunks.iter() {
        let path = config.chunk_path(chunk_pos.0);
        if path.exists() {
            match registry.load(&mut commands, entity, &config, chunk_pos.0) {
                Ok(()) => {
                    commands.entity(entity).insert(ChunkLoadedFromDisk);
                }
                Err(e) => {
                    error!("Failed to auto-load chunk {:?}: {:?}", chunk_pos.0, e);
                    // Still send generation event on load failure
                    commands.trigger(ChunkNeedsGeneration {
                        entity,
                        pos: chunk_pos.0,
                    });
                }
            }
        } else {
            // No saved data - needs generation
            commands.trigger(ChunkNeedsGeneration {
                entity,
                pos: chunk_pos.0,
            });
        }
    }
}

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashMap,
    prelude::*,
};

#[derive(Component)]
#[require(ChunkPosition, Visibility)]
#[component(
    immutable,
    on_add= on_add_chunk,
    on_remove = on_remove_chunk
)]
pub struct Chunk;

/// Registers the chunk with [`ChunkManager`] when added.
fn on_add_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap().0;
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

/// Unregisters the chunk from [`ChunkManager`] when removed.
fn on_remove_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap().0;
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
#[require(Transform)]
#[component(
    immutable,
    on_add= on_add_chunk_pos,
)]
pub struct ChunkPosition(pub IVec3);

/// Sets the entity's [`Transform`] translation based on chunk position and size.
fn on_add_chunk_pos(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap();
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
#[derive(Resource)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct ChunkManager {
    chunk_size: Vec3,
    chunks: HashMap<IVec3, Entity>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE)
    }
}

impl ChunkManager {
    pub const DEFAULT_SIZE: Vec3 = vec3(10.0, 10.0, 10.0);

    /// Creates a new chunk manager with the specified chunk size.
    pub fn new(chunk_size: Vec3) -> Self {
        Self {
            chunk_size,
            chunks: default(),
        }
    }

    /// Returns the size of chunks in world units.
    pub fn get_size(&self) -> Vec3 {
        self.chunk_size
    }

    /// Inserts a new chunk into the manager.
    ///
    /// Returns the previous chunk entity if one already existed at this position.
    ///
    /// Note: Called automatically when a [`Chunk`] component is added.
    pub fn insert(&mut self, pos: IVec3, id: Entity) -> Option<Entity> {
        self.chunks.insert(pos, id)
    }

    /// Removes a chunk from the manager.
    ///
    /// Returns the chunk's entity if it existed.
    ///
    /// Note: Called automatically when a [`Chunk`] component is removed.
    pub fn remove(&mut self, pos: &IVec3) -> Option<Entity> {
        self.chunks.remove(pos)
    }

    /// Converts world coordinates into chunk position.
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

    /// Gets the chunk entity at the specified chunk position if it exists.
    pub fn get_chunk(&self, chunk_pos: &IVec3) -> Option<Entity> {
        self.chunks.get(chunk_pos).copied()
    }

    /// Gets the chunk entity at the specified world position if it exists.
    pub fn get_chunk_form_pos(&self, pos: &Vec3) -> Option<Entity> {
        self.get_chunk(&self.get_chunk_pos(pos))
    }

    /// Checks if a chunk is loaded at the specified chunk position.
    pub fn is_loaded(&self, chunk_pos: &IVec3) -> bool {
        self.chunks.contains_key(chunk_pos)
    }
}

pub trait SetCunkyChunkSize {
    fn set_chunky_chunk_size(&mut self, size: Vec3) -> &mut Self;
}

impl SetCunkyChunkSize for App {
    fn set_chunky_chunk_size(&mut self, size: Vec3) -> &mut Self {
        self.insert_resource(ChunkManager::new(size))
    }
}

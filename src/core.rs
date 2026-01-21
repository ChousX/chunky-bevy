use std::marker::PhantomData;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashMap,
    prelude::*,
};

pub trait ChunkManaging {
    ///Dimensions of each chunk being managed
    const DIMENSIONS: Vec3 = Vec3::splat(10.0);
}

#[derive(Resource)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct ChunkManagerResource<T: ChunkManaging> {
    _type: PhantomData<T>,
    pub entity: Entity,
}

impl<T: ChunkManaging> ChunkManagerResource<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            _type: PhantomData,
            entity,
        }
    }
}

impl<T: ChunkManaging> FromWorld for ChunkManagerResource<T> {
    fn from_world(world: &mut World) -> Self {
        let id = world
            .commands()
            .spawn(ChunkManager::new(T::DIMENSIONS))
            .id();
        Self::new(id)
    }
}

#[derive(Component, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct ChunkManager {
    chunk_size: Vec3,
    chunks: HashMap<IVec3, Entity>,
}

impl ChunkManager {
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

#[derive(Component)]
#[component(
    immutable,
    on_add= on_add_chunk,
    on_remove = on_remove_chunk
)]
#[require(ChunkPosition)]
pub struct Chunk(pub Entity);

#[derive(Component, Default, Deref, DerefMut)]
#[require(Transform)]
#[component(
    immutable,
    on_add= on_add_chunk_pos,
)]
pub struct ChunkPosition(pub IVec3);

/// Sets the entity's [`Transform`] translation based on chunk position and size.
fn on_add_chunk_pos(
    mut world: bevy::ecs::world::DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap();
    let &Chunk(chunk_manager_id) = world.get::<Chunk>(entity).unwrap();
    let chunk_size = world
        .get::<ChunkManager>(chunk_manager_id)
        .unwrap()
        .chunk_size;
    let translation = chunk_pos.as_vec3() * chunk_size;
    world.get_mut::<Transform>(entity).unwrap().translation = translation;
}

/// Registers the chunk with [`ChunkManager`] when added.
fn on_add_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap().0;
    let &Chunk(chunk_manager_id) = world.get::<Chunk>(entity).unwrap();
    let mut chunk_manager = world.get_mut::<ChunkManager>(chunk_manager_id).unwrap();
    if chunk_manager.is_loaded(&chunk_pos) {
        error!(
            "New chunk at pos:{} was not spawned there was already a chunk there",
            chunk_pos
        );
        return;
    }
    chunk_manager.insert(chunk_pos, entity);
}

/// Unregisters the chunk from [`ChunkManager`] when removed.
fn on_remove_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPosition>(entity).unwrap().0;
    let &Chunk(chunk_manager_id) = world.get::<Chunk>(entity).unwrap();
    world
        .get_mut::<ChunkManager>(chunk_manager_id)
        .unwrap()
        .remove(&chunk_pos);
}

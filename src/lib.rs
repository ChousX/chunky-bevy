use std::collections::HashMap;
use bevy::{
    ecs::{
        component::HookContext,
        query::{QueryData, QueryFilter},
        world::DeferredWorld,
    },
    prelude::*,
};

#[derive(Component)]
#[require(ChunkPos, Visibility)]
#[component(
    immutable,
    on_add= on_add_chunk,
    on_remove = on_remove_chunk
)]
pub struct Chunk;

///Adds Chunk to ChunkManager
fn on_add_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext){
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    world
        .get_resource_mut::<ChunkManager>()
        .unwrap()
        .insert(chunk_pos, entity);
}

///Removes Chunk from ChunkManager
fn on_remove_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    world
        .get_resource_mut::<ChunkManager>()
        .unwrap()
        .remove(chunk_pos);
}

#[derive(Component, Default, Deref, DerefMut)]
#[require(Transform)]
#[component(
    immutable,
    on_add= on_add_chunk_pos,
)]
pub struct ChunkPos(pub IVec3);

///Updates Transform to match ChunkPos
fn on_add_chunk_pos(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let translation = world.get::<ChunkPos>(entity).unwrap().into_vec3();
    world.get_mut::<Transform>(entity).unwrap().translation = translation;
}

pub struct ChunkManager{
    chunk_size: Vec3,
    chunks: HashMap<IVec3, Entity>
}

impl ChunkManager{
    //Inserts a new chunk into manager
    //Returns the previous chunk id if there was one
    pub fn insert(&mut self, pos: IVec3, id: Entity) -> Option<Entity> {
        self.chunks.insert(pos, id)
    }

    ///Remove Chunk from ChunkManager
    pub fn remove(&mut self, pos: IVec3) -> Option<Entity> {
        self.chunks.remove(pos)
    }
}

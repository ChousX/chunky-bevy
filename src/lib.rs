use bevy::prelude::*;

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
}

impl ChunkManager{
    //Inserts a new chunk into manager
    //Returns the previous chunk id if there was one
    pub fn insert(&mut self, pos: IVec3, id: Entity) -> Option<Entity> {
        let IVec3 { x, y, z } = pos;
        //the or_default seems wrong to me
        self.0.entry(z).or_default().insert(ivec2(x, y), id)
    }
    ///Remove Chunk from ChunkManager
    pub fn remove(&mut self, pos: IVec3) -> Option<Entity> {
        let IVec3 { x, y, z } = pos;
        self.0.get_mut(&z)?.remove(&ivec2(x, y))
    }
}

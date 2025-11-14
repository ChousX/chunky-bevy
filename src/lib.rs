use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use std::collections::HashMap;
pub mod prelude {
    pub use crate::{Chunk, ChunkLoader, ChunkManager, ChunkPos, ChunkyPlugin};
}

pub struct ChunkyPlugin {
    chunk_size: Vec3,
}

impl Plugin for ChunkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkManager::new(self.chunk_size))
            .init_state::<ChunkBoundryVisualizer>()
            .add_systems(
                Update,
                chunk_boundry_visualizer.run_if(in_state(ChunkBoundryVisualizer::On)),
            );
        app.add_systems(Update, chunk_loader);
    }
}

impl ChunkyPlugin {
    pub const THREE_DIMETION: Self = Self {
        chunk_size: vec3(10.0, 10.0, 10.0),
    };
    //ToDo: Make 2d usable
    //pub const TWO_DIMENTION: Self = Self {chunk_size: vec3(10.0, 1.0, 10.0)};
}

impl Default for ChunkyPlugin {
    fn default() -> Self {
        Self::THREE_DIMETION
    }
}

#[derive(Component)]
#[require(ChunkPos, Visibility)]
#[component(
    immutable,
    on_add= on_add_chunk,
    on_remove = on_remove_chunk
)]
pub struct Chunk;

///Adds Chunk to ChunkManager
fn on_add_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    if let Some(_) = world
        .get_resource_mut::<ChunkManager>()
        .unwrap()
        .insert(chunk_pos, entity)
    {
        //if we remove it then the chunk just added will also be removed so...
        warn!("a chunk is being lost")
    }
    #[cfg(feature = "chunk_info")]
    info!("[ChunkInfo]ChunkPos: {chunk_pos:?}");
}

///Removes Chunk from ChunkManager
fn on_remove_chunk(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap().0;
    world
        .get_resource_mut::<ChunkManager>()
        .unwrap()
        .remove(&chunk_pos);
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
    let chunk_pos = world.get::<ChunkPos>(entity).unwrap();
    let chunk_size = world.get_resource::<ChunkManager>().unwrap().chunk_size;
    let translation = chunk_pos.as_vec3() * chunk_size;
    world.get_mut::<Transform>(entity).unwrap().translation = translation;
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    chunk_size: Vec3,
    chunks: HashMap<IVec3, Entity>,
}

impl ChunkManager {
    ///New chunk manager
    pub fn new(chunk_size: Vec3) -> Self {
        Self {
            chunk_size,
            chunks: default(),
        }
    }

    pub fn get_size(&self) -> Vec3 {
        self.chunk_size
    }

    ///Inserts a new chunk into manager
    ///Returns the previous chunk id if there was one
    pub fn insert(&mut self, pos: IVec3, id: Entity) -> Option<Entity> {
        self.chunks.insert(pos, id)
    }

    ///Remove Chunk from ChunkManager
    pub fn remove(&mut self, pos: &IVec3) -> Option<Entity> {
        self.chunks.remove(pos)
    }

    ///Converts cordinits into chunk position
    pub fn get_chunk_pos(&self, pos: &Vec3) -> IVec3 {
        (*pos / self.chunk_size).floor().as_ivec3()
    }

    ///Gets chunk entity if it exists
    pub fn get_chunk(&self, chunk_pos: &IVec3) -> Option<Entity> {
        self.chunks.get(chunk_pos).copied()
    }

    ///Gets chunk entity form world cordinits if it exists
    pub fn get_chunk_form_pos(&self, pos: &Vec3) -> Option<Entity> {
        self.get_chunk(&self.get_chunk_pos(pos))
    }

    pub fn is_loaded(&self, chunk_pos: &IVec3) -> bool {
        self.chunks.contains_key(chunk_pos)
    }
}

/// ChunkLoader(0, 0, 0) would generate one chunk derctly under the entity.
/// ChunkLoader(1, 1, 1,) would generate a 3x3x3 cube where the senter chunk is the one derctly
/// under the entity.
#[derive(Component, Default, Debug)]
pub struct ChunkLoader(pub IVec3);

///Load Chunks Around ChunkLoader
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ChunkBoundryVisualizer {
    #[default]
    On,
    Off,
}

///Shows all existing chunk boundreis
///p011 ------ p111
/// /|          /|
///p001 ----- p101
/// | p010 ----| p110
/// |/         |/
///p000 ----- p100
fn chunk_boundry_visualizer(
    chunk_manager: Res<ChunkManager>,
    chunks: Query<&ChunkPos>,
    mut gizmos: Gizmos,
) {
    let chunk_size = chunk_manager.get_size(); // Vec3

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

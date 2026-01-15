use bevy::prelude::*;

use crate::prelude::{Chunk, ChunkManager, ChunkPositon};
pub struct ChunkLoaderPlugin;
impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, chunk_loader);
        #[cfg(feature = "reflect")]
        app.register_type::<ChunkLoader>();
    }
}

#[derive(Component, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct ChunkLoader {
    pub radius: IVec3,
    pub chunk_manager_id: Entity,
}

/// Load Chunks Around ChunkLoader
fn chunk_loader(
    chunks: Query<(&ChunkLoader, &GlobalTransform)>,
    chunk_managers: Query<&mut ChunkManager>,
    mut commands: Commands,
) {
    for (
        &ChunkLoader {
            radius,
            chunk_manager_id,
        },
        g_transform,
    ) in chunks.iter()
    {
        let chunk_manager = chunk_managers
            .get(chunk_manager_id)
            .expect("Missing ChunkManager for ChunkLoader");
        let translation = g_transform.translation();
        let in_chunk = chunk_manager.get_chunk_pos(&translation);
        for x in -radius.x..=radius.x {
            for y in -radius.y..=radius.y {
                for z in -radius.z..=radius.z {
                    let target_chunk = in_chunk + ivec3(x, y, z);
                    if !chunk_manager.is_loaded(&target_chunk) {
                        commands.spawn((Chunk(chunk_manager_id), ChunkPositon(target_chunk)));
                    }
                }
            }
        }
    }
}

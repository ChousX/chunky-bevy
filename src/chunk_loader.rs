use std::path::Path;

use bevy::prelude::*;

use crate::{Chunk, ChunkManager, ChunkPos};
pub struct ChunkLoaderPlugin;
impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, chunk_loader);
        #[cfg(feature = "reflect")]
        app.register_type::<ChunkLoader>();
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
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

#[derive(Resource, Debug)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct ChunkLoaderSettings {
    /// max_loaded: 0 will mean do not despawn chunks based on max_loaded amount
    pub max_loaded: usize,
}

//! Utility functions for spawning chunks in bulk.

use crate::core::{Chunk, ChunkPos};
use bevy::prelude::*;

/// Spawns chunks in a rectangular region defined by two chunk positions.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::helpers::*;
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

/// Spawns chunks covering integer chunk positions from floored `pos_0` to floored `pos_1`.
///
/// **Note**: This treats the input positions as chunk coordinates after flooring,
/// not as world positions. Use [`crate::ChunkManager::get_chunk_pos`] to convert world
/// positions to chunk positions first if needed.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::helpers::*;
///
/// fn setup(mut commands: Commands) {
///     // Spawns chunks at positions (0,0,0) through (5,2,5)
///     spawn_chunks_rect_from_world_pos(
///         &mut commands,
///         Vec3::new(0.5, 0.0, 0.5),
///         Vec3::new(5.9, 2.1, 5.9)
///     );
/// }
/// ```
pub fn spawn_chunks_rect_from_world_pos(commands: &mut Commands, pos_0: Vec3, pos_1: Vec3) {
    let chunk_pos_0 = pos_0.floor().as_ivec3();
    let chunk_pos_1 = pos_1.floor().as_ivec3();
    spawn_chunks_rect(commands, chunk_pos_0, chunk_pos_1);
}

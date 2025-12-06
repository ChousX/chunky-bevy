use crate::ChunkManager;
use crate::ChunkPos;
use bevy::prelude::*;

pub struct ChunkBoundryVisualizerPlugin;
impl Plugin for ChunkBoundryVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChunkBoundryVisualizer>().add_systems(
            Update,
            chunk_boundry_visualizer.run_if(in_state(ChunkBoundryVisualizer::On)),
        );
        #[cfg(feature = "reflect")]
        app.register_type::<ChunkBoundryVisualizer>();
    }
}

/// State for controlling chunk boundary visualization
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Hash))]
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

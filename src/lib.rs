#[cfg(feature = "chunk_loader")]
mod chunk_loader;
#[cfg(feature = "chunk_saver")]
mod chunk_saver;
#[cfg(feature = "chunk_unloader")]
mod chunk_unloader;
#[cfg(feature = "chunk_visualizer")]
mod chunk_visualizer;

mod core;

/// Utility functions for spawning chunks in bulk
pub mod helpers;

use bevy::prelude::*;

/// Re-exports of commonly used types
pub mod prelude {
    pub use crate::ChunkyPlugin;
    #[cfg(feature = "chunk_loader")]
    pub use crate::chunk_loader::ChunkLoader;
    #[cfg(feature = "chunk_saver")]
    pub use crate::chunk_saver::{
        ChunkDataRegistry, ChunkSaveConfig, ChunkSavingPlugin, RegisterChunkData, SaveError,
        SaveStyle,
    };
    #[cfg(all(feature = "chunk_unloader", feature = "chunk_loader"))]
    pub use crate::chunk_unloader::ChunkUnloadRadius;
    #[cfg(feature = "chunk_unloader")]
    pub use crate::chunk_unloader::{
        ChunkLastAccess, ChunkPinned, ChunkUnloadByDistance, ChunkUnloadEvent, ChunkUnloadLimit,
        ChunkUnloadReason,
    };
    #[cfg(feature = "chunk_visualizer")]
    pub use crate::chunk_visualizer::ChunkBoundryVisualizer;
    pub use crate::core::{Chunk, ChunkManager, ChunkManagerResource, ChunkManaging, ChunkPositon};
}
pub struct ChunkyPlugin;

impl Plugin for ChunkyPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "chunk_loader")]
        app.add_plugins(chunk_loader::ChunkLoaderPlugin);
        #[cfg(feature = "chunk_visualizer")]
        app.add_plugins(chunk_visualizer::ChunkBoundryVisualizerPlugin);
        #[cfg(feature = "chunk_unloader")]
        app.add_plugins(chunk_unloader::ChunkUnloaderPlugin);
        #[cfg(feature = "reflect")]
        app.register_type::<ChunkPos>()
            .register_type::<ChunkManager>();
    }
}

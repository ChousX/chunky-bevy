//! A simple and efficient chunk management system for Bevy.
//!
//! # Quick Start
//!
//! ```no_run
//! use bevy::prelude::*;
//! use chunky_bevy::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(ChunkyPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Spawn a chunk loader that generates chunks around it
//!     commands.spawn((
//!         Transform::default(),
//!         ChunkLoader(IVec3::new(2, 1, 2)), // Load 5x3x5 chunks
//!     ));
//! }
//! ```
//!
//! # Features
//!
//! - `chunk_visualizer` (default) - Enables debug visualization of chunk boundaries
//! - `chunk_loader` (default) - Enables automatic chunk loading around ChunkLoader entities
//! - `chunk_info` - Logs chunk spawn/despawn events

#[cfg(feature = "chunk_loader")]
mod chunk_loader;
#[cfg(feature = "chunk_saver")]
mod chunk_saver;
#[cfg(feature = "chunk_unloader")]
mod chunk_unloader;
#[cfg(feature = "chunk_visualizer")]
mod chunk_visualizer;

/// Core Types and Logic
pub mod core;
/// Utility functions for spawning chunks in bulk
pub mod helpers;

use bevy::prelude::*;

/// Re-exports of commonly used types
pub mod prelude {
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
    pub use crate::{
        ChunkyPlugin,
        core::{Chunk, ChunkManager, ChunkPosition, SetCunkyChunkSize},
    };
}

/// The main plugin for chunk management.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use chunky_bevy::ChunkyPlugin;
///
/// App::new()
///     .add_plugins(ChunkyPlugin::default()) // 10x10x10 chunks
///     .run();
/// ```
pub struct ChunkyPlugin;

impl Plugin for ChunkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(core::ChunkManager::default());
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

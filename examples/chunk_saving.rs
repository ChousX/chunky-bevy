//! ===========================================================================
//! chunk_saving.rs - Manual chunk saving and loading
//! ===========================================================================
//! Press S to save all chunks, L to load, D to delete save files.

use bevy::prelude::*;
use chunky_bevy::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkyPlugin)
        .add_plugins(ChunkSavingPlugin::new("saves/test_world"))
        .init_resource::<ChunkManagerResource<WorldChunks>>()
        .register_chunk_data::<VoxelData>()
        .register_chunk_data::<ChunkMetadata>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, show_chunks))
        .run();
}

struct WorldChunks;
impl ChunkManaging for WorldChunks {
    const DIMENSIONS: Vec3 = Vec3::splat(10.0);
}

/// Example voxel data component
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
struct VoxelData {
    values: Vec<f32>,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            values: vec![0.0; 8], // Small for demo
        }
    }
}

/// Example metadata component
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
struct ChunkMetadata {
    biome: String,
    generated_at: u64,
}

fn setup(mut commands: Commands, chunk_manager_resource: Res<ChunkManagerResource<WorldChunks>>) {
    // Spawn a few chunks with data
    for x in -1..=1 {
        for z in -1..=1 {
            let pos = IVec3::new(x, 0, z);
            commands.spawn((
                Chunk(chunk_manager_resource.entity),
                ChunkPosition(pos),
                VoxelData {
                    values: vec![(x + z) as f32; 8],
                },
                ChunkMetadata {
                    biome: format!("biome_{x}_{z}"),
                    generated_at: 12345,
                },
            ));
        }
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    info!("Press S to save, L to load, D to delete saves, C to clear chunks");
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    world: &World,
    mut commands: Commands,
    chunks: Query<
        (
            Entity,
            &ChunkPosition,
            Option<&VoxelData>,
            Option<&ChunkMetadata>,
        ),
        With<Chunk>,
    >,
    registry: Res<ChunkDataRegistry>,
    config: Res<ChunkSaveConfig>,
    chunk_manager_resource: Res<ChunkManagerResource<WorldChunks>>,
) {
    // Save all chunks
    if keys.just_pressed(KeyCode::KeyS) {
        let mut saved = 0;
        for (entity, pos, _, _) in chunks.iter() {
            match registry.save(world, entity, &config) {
                Ok(_) => {
                    info!("Saved chunk {:?}", pos.0);
                    saved += 1;
                }
                Err(e) => error!("Failed to save chunk {:?}: {:?}", pos.0, e),
            }
        }
        info!("Saved {} chunks", saved);
    }

    // Load chunks (spawns new entities)
    if keys.just_pressed(KeyCode::KeyL) {
        for x in -1..=1 {
            for z in -1..=1 {
                let pos = IVec3::new(x, 0, z);
                let entity = commands
                    .spawn((Chunk(chunk_manager_resource.entity), ChunkPosition(pos)))
                    .id();
                match registry.load(&mut commands, entity, &config, pos) {
                    Ok(_) => info!("Loaded chunk {:?}", pos),
                    Err(e) => warn!("No save for chunk {:?}: {:?}", pos, e),
                }
            }
        }
    }

    // Delete save files
    if keys.just_pressed(KeyCode::KeyD) {
        if let Err(e) = std::fs::remove_dir_all(&config.base_path) {
            warn!("Could not delete saves: {e}");
        } else {
            info!("Deleted save directory");
        }
    }

    // Clear all chunks
    if keys.just_pressed(KeyCode::KeyC) {
        for (entity, _, _, _) in chunks.iter() {
            commands.entity(entity).despawn();
        }
        info!("Cleared all chunks");
    }
}

fn show_chunks(
    chunks: Query<(&ChunkPosition, Option<&VoxelData>, Option<&ChunkMetadata>), With<Chunk>>,
    mut gizmos: Gizmos,
) {
    for (pos, voxel, meta) in chunks.iter() {
        let world_pos = pos.0.as_vec3() * 10.0 + Vec3::splat(5.0);

        // Draw chunk as a cube
        let color = if voxel.is_some() && meta.is_some() {
            Color::srgb(0.0, 1.0, 0.0) // Green = has data
        } else {
            Color::srgb(1.0, 0.0, 0.0) // Red = no data
        };

        gizmos.cube(
            Transform::from_translation(world_pos).with_scale(Vec3::splat(9.0)),
            color,
        );
    }
}

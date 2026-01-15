//! ===========================================================================
//! chunk_saving_super.rs - SuperChunk saving (multiple chunks per file)
//! ===========================================================================
//! Tests batch save/load with grouped chunk files.

use bevy::prelude::*;
use chunky_bevy::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkyPlugin)
        .add_plugins(ChunkSavingPlugin::new("saves/super_world").with_style(
            SaveStyle::SuperChunk {
                size: UVec3::splat(4),
            },
        ))
        .init_resource::<ChunkManagerResource<SuperWorldChunks>>()
        .register_chunk_data::<SimpleData>()
        .add_systems(Startup, setup_super)
        .add_systems(Update, (handle_input_super, visualize_chunks_super))
        .run();
}

struct SuperWorldChunks;
impl ChunkManaging for SuperWorldChunks {
    const SIZE: Vec3 = Vec3::splat(10.0);
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
struct SimpleData {
    id: i32,
}

fn setup_super(
    mut commands: Commands,
    chunk_manager_resource: Res<ChunkManagerResource<SuperWorldChunks>>,
) {
    // Spawn a grid of chunks
    for x in -4..=4 {
        for z in -4..=4 {
            let pos = IVec3::new(x, 0, z);
            commands.spawn((
                Chunk(chunk_manager_resource.entity),
                ChunkPositon(pos),
                SimpleData { id: x * 100 + z },
            ));
        }
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 120.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    info!("SuperChunk mode: 4x4x4 chunks per file");
    info!("Press S for batch save, L for batch load, C to clear, I to inspect files");
}

fn handle_input_super(
    keys: Res<ButtonInput<KeyCode>>,
    world: &World,
    mut commands: Commands,
    chunks: Query<Entity, With<Chunk>>,
    registry: Res<ChunkDataRegistry>,
    config: Res<ChunkSaveConfig>,
    chunk_manager_resource: Res<ChunkManagerResource<SuperWorldChunks>>,
) {
    // Batch save all chunks
    if keys.just_pressed(KeyCode::KeyS) {
        let entities: Vec<_> = chunks.iter().collect();
        match registry.save_batch(world, &entities, &config) {
            Ok(_) => info!("Batch saved {} chunks", entities.len()),
            Err(e) => error!("Batch save failed: {:?}", e),
        }
    }

    // Batch load from super-chunk at origin
    if keys.just_pressed(KeyCode::KeyL) {
        // Load super-chunks that cover our area
        for sx in -1..=1 {
            for sz in -1..=1 {
                // Any position in the super-chunk region works
                let pos = IVec3::new(sx * 4, 0, sz * 4);
                match registry.load_batch(
                    &mut commands,
                    &config,
                    pos,
                    chunk_manager_resource.entity,
                ) {
                    Ok(loaded) => {
                        info!(
                            "Loaded {} chunks from super-chunk ({}, {})",
                            loaded.len(),
                            sx,
                            sz
                        );
                    }
                    Err(e) => warn!("No super-chunk at ({}, {}): {:?}", sx, sz, e),
                }
            }
        }
    }

    // Clear all chunks
    if keys.just_pressed(KeyCode::KeyC) {
        for entity in chunks.iter() {
            commands.entity(entity).despawn();
        }
        info!("Cleared all chunks");
    }

    // Inspect save directory
    if keys.just_pressed(KeyCode::KeyI) {
        match std::fs::read_dir(&config.base_path) {
            Ok(entries) => {
                info!("Save files:");
                for entry in entries.flatten() {
                    let path = entry.path();
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    info!("  {:?} ({} bytes)", path.file_name().unwrap(), size);
                }
            }
            Err(_) => info!("No save directory yet"),
        }
    }
}

fn visualize_chunks_super(
    chunks: Query<(&ChunkPositon, Option<&SimpleData>), With<Chunk>>,
    config: Res<ChunkSaveConfig>,
    mut gizmos: Gizmos,
) {
    let chunk_size = 10.0;

    // Draw each chunk
    for (pos, data) in chunks.iter() {
        let world_pos = pos.0.as_vec3() * chunk_size + Vec3::new(5.0, 0.0, 5.0);

        // Color based on whether it has data
        let color = if data.is_some() {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.2, 0.2)
        };

        gizmos.rect(
            Isometry3d::new(
                world_pos,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            Vec2::splat(chunk_size - 0.5),
            color,
        );
    }

    // Draw super-chunk boundaries
    if let SaveStyle::SuperChunk { size } = &config.style {
        let super_size = size.as_vec3() * chunk_size;

        for sx in -1..=1 {
            for sz in -1..=1 {
                let super_pos = Vec3::new(
                    sx as f32 * super_size.x + super_size.x / 2.0,
                    0.5,
                    sz as f32 * super_size.z + super_size.z / 2.0,
                );

                // Yellow outline for super-chunk boundaries
                gizmos.rect(
                    Isometry3d::new(
                        super_pos,
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    ),
                    Vec2::new(super_size.x - 0.2, super_size.z - 0.2),
                    Color::srgb(1.0, 1.0, 0.0),
                );
            }
        }
    }
}

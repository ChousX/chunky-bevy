//! Example: Automatic chunk saving/loading with ChunkLoader and ChunkUnloader
//!
//! Move the camera with WASD. Chunks auto-save when unloaded and auto-load when spawned.

use bevy::prelude::*;
use chunky_bevy::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkyPlugin::default())
        .add_plugins(
            ChunkSavingPlugin::new("saves/auto_world")
                .with_auto_save()
                .with_auto_load(),
        )
        .register_chunk_data::<TerrainData>()
        // Enable unloading
        .insert_resource(ChunkUnloadLimit { max_chunks: 50 })
        .insert_resource(ChunkUnloadByDistance)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_loader, show_info, visualize_chunks))
        .run();
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
struct TerrainData {
    height: f32,
    moisture: f32,
}

fn setup(mut commands: Commands) {
    // Camera looking down
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 100.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Chunk loader that moves with input
    commands.spawn((
        Transform::default(),
        ChunkLoader(IVec3::new(3, 0, 3)),
        ChunkUnloadRadius(IVec3::new(5, 0, 5)),
        Mover,
    ));

    info!("WASD to move loader. Chunks auto-save on unload, auto-load on spawn.");
}

#[derive(Component)]
struct Mover;

fn move_loader(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut movers: Query<&mut Transform, With<Mover>>,
) {
    let speed = 30.0;
    let mut dir = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        dir.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }

    if dir != Vec3::ZERO {
        let delta = dir.normalize() * speed * time.delta_secs();
        for mut transform in movers.iter_mut() {
            transform.translation += delta;
        }
    }
}

fn show_info(
    chunks: Query<&ChunkPos, With<Chunk>>,
    movers: Query<&Transform, With<Mover>>,
    chunk_manager: Res<ChunkManager>,
) {
    // Log occasionally
    static mut FRAME: u32 = 0;
    unsafe {
        FRAME += 1;
        if FRAME % 120 != 0 {
            return;
        }
    }

    let count = chunks.iter().count();
    if let Ok(transform) = movers.single() {
        let loader_chunk = chunk_manager.get_chunk_pos(&transform.translation);
        info!(
            "Loader at chunk {:?}, {} chunks loaded",
            loader_chunk, count
        );
    }
}

fn visualize_chunks(
    chunks: Query<(&ChunkPos, Option<&TerrainData>), With<Chunk>>,
    movers: Query<&Transform, With<Mover>>,
    mut gizmos: Gizmos,
) {
    let chunk_size = 10.0;

    for (pos, terrain) in chunks.iter() {
        let world_pos = pos.0.as_vec3() * chunk_size + Vec3::new(5.0, 0.0, 5.0);

        let color = if terrain.is_some() {
            Color::srgb(0.2, 0.8, 0.2)
        } else {
            Color::srgb(0.5, 0.5, 0.5)
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

    // Show loader position
    if let Ok(transform) = movers.single() {
        gizmos.sphere(
            Isometry3d::from_translation(transform.translation),
            2.0,
            Color::srgb(1.0, 1.0, 0.0),
        );
    }
}

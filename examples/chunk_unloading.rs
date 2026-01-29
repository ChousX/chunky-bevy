//! Demonstrates chunk unloading strategies.
//!
//! Move around with WASD/QE and watch chunks load/unload based on distance.
//! The UI shows chunk statistics and which unload strategy is active.
//!
//! Controls:
//! - WASD: Move horizontally
//! - Q/E: Move down/up
//! - Space: Cycle through unload strategies
//! - Right-click + drag: Look around

use bevy::{input::mouse::MouseMotion, prelude::*};
use chunky_bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ChunkyPlugin::default())
        .init_state::<UnloadStrategy>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                fly_camera,
                cycle_unload_strategy,
                update_unload_resources,
                update_ui,
                log_unload_events,
            ),
        )
        .run();
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum UnloadStrategy {
    #[default]
    None,
    DistanceOnly,
    LimitOnly,
    Hybrid,
}

impl UnloadStrategy {
    fn next(self) -> Self {
        match self {
            Self::None => Self::DistanceOnly,
            Self::DistanceOnly => Self::LimitOnly,
            Self::LimitOnly => Self::Hybrid,
            Self::Hybrid => Self::None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::None => "None (chunks never unload)",
            Self::DistanceOnly => "Distance Only (unload when out of range)",
            Self::LimitOnly => "Limit Only (LRU eviction at 50 chunks)",
            Self::Hybrid => "Hybrid (out of range AND over limit)",
        }
    }
}

#[derive(Component)]
struct FlyCam {
    speed: f32,
    sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            speed: 30.0,
            sensitivity: 0.003,
            pitch: -0.3,
            yaw: 0.0,
        }
    }
}

#[derive(Component)]
struct StatsText;

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands, mut visualizer: ResMut<NextState<ChunkBoundryVisualizer>>) {
    // Camera with fly controls and chunk loading
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam::default(),
        Player,
        ChunkLoader(IVec3::new(3, 1, 3)),
        ChunkUnloadRadius(IVec3::new(5, 2, 5)),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // UI
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        StatsText,
    ));

    visualizer.set(ChunkBoundryVisualizer::On);
}

fn fly_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCam)>,
) {
    let Ok((mut transform, mut cam)) = query.single_mut() else {
        return;
    };

    // Mouse look (right-click held)
    if mouse_buttons.pressed(MouseButton::Right) {
        for motion in mouse_motion.read() {
            cam.yaw -= motion.delta.x * cam.sensitivity;
            cam.pitch -= motion.delta.y * cam.sensitivity;
            cam.pitch = cam.pitch.clamp(-1.5, 1.5);
        }
    } else {
        mouse_motion.clear();
    }

    transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);

    // Keyboard movement
    let mut velocity = Vec3::ZERO;
    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();

    if keyboard.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keyboard.pressed(KeyCode::KeyE) {
        velocity += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        velocity -= Vec3::Y;
    }

    if velocity != Vec3::ZERO {
        transform.translation += velocity.normalize() * cam.speed * time.delta_secs();
    }
}

fn cycle_unload_strategy(
    keyboard: Res<ButtonInput<KeyCode>>,
    current: Res<State<UnloadStrategy>>,
    mut next: ResMut<NextState<UnloadStrategy>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next.set(current.get().next());
    }
}

fn update_unload_resources(
    mut commands: Commands,
    strategy: Res<State<UnloadStrategy>>,
    distance_res: Option<Res<ChunkUnloadByDistance>>,
    limit_res: Option<Res<ChunkUnloadLimit>>,
) {
    if !strategy.is_changed() {
        return;
    }

    // Remove existing resources
    if distance_res.is_some() {
        commands.remove_resource::<ChunkUnloadByDistance>();
    }
    if limit_res.is_some() {
        commands.remove_resource::<ChunkUnloadLimit>();
    }

    // Insert based on new strategy
    match strategy.get() {
        UnloadStrategy::None => {}
        UnloadStrategy::DistanceOnly => {
            commands.insert_resource(ChunkUnloadByDistance);
        }
        UnloadStrategy::LimitOnly => {
            commands.insert_resource(ChunkUnloadLimit { max_chunks: 50 });
        }
        UnloadStrategy::Hybrid => {
            commands.insert_resource(ChunkUnloadByDistance);
            commands.insert_resource(ChunkUnloadLimit { max_chunks: 50 });
        }
    }
}

fn update_ui(
    mut text_query: Query<&mut Text, With<StatsText>>,
    chunks: Query<&ChunkPosition, With<Chunk>>,
    pinned: Query<(), With<ChunkPinned>>,
    player: Query<&Transform, With<Player>>,
    strategy: Res<State<UnloadStrategy>>,
    chunk_manager: Res<ChunkManager>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        return;
    };

    let chunk_count = chunks.iter().count();
    let pinned_count = pinned.iter().count();
    let player_chunk = chunk_manager.get_chunk_pos(&player_transform.translation);

    **text = format!(
        "Chunks: {} (pinned: {})\n\
         Player chunk: {}\n\
         \n\
         Strategy: {}\n\
         \n\
         [Space] Cycle strategy\n\
         [WASD/QE] Move\n\
         [Right-click + drag] Look",
        chunk_count,
        pinned_count,
        player_chunk,
        strategy.get().label(),
    );
}

fn log_unload_events(mut events: MessageReader<ChunkUnloadEvent>) {
    for event in events.read() {
        info!("Chunk {:?} unloaded: {:?}", event.chunk_pos, event.reason);
    }
}

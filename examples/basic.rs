use bevy::{input::mouse::MouseMotion, prelude::*};
use chunky::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ChunkyPlugin::default());
    app.add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, camera_look, cube_movement));
    app.run();
}

#[derive(Component, Debug)]
struct MainCamera;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 5.0, 0.0),
    ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        ChunkLoader::default(),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = query.single_mut().unwrap();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyE) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::KeyQ) {
        direction -= Vec3::Y;
    }

    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * 10.0 * time.delta_secs();
    }
}

fn camera_look(
    mut motion_evr: MessageReader<MouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut transform = query.single_mut().unwrap();

    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    const SENSITIVITY: f32 = 0.005;
    let yaw = -delta.x * SENSITIVITY;
    let pitch = -delta.y * SENSITIVITY;

    let rotation = Quat::from_rotation_y(yaw) * transform.rotation;
    let new_rotation = Quat::from_rotation_x(pitch).mul_quat(rotation);

    // Prevent flipping
    let up = new_rotation * Vec3::Y;
    if up.y.abs() > 0.1 {
        transform.rotation = new_rotation;
    }
}

fn cube_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Mesh3d>>,
) {
    let mut transform = query.single_mut().unwrap();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyK) {
        direction += transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyJ) {
        direction -= transform.forward().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyH) {
        direction -= transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyL) {
        direction += transform.right().as_vec3();
    }
    if keyboard.pressed(KeyCode::KeyY) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::KeyI) {
        direction -= Vec3::Y;
    }

    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * 10.0 * time.delta_secs();
        info!("cube_pos:{}", transform.translation);
    }
}

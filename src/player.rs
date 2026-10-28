use avian3d::prelude::*;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

const PLAYER_RADIUS: f32 = 0.35;
const PLAYER_LENGTH: f32 = 1.1;
const PLAYER_SPEED: f32 = 7.0;
const JUMP_SPEED: f32 = 7.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const MAX_PITCH: f32 = 1.54;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component, Default)]
struct PlayerLook {
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
struct Grounded;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                toggle_mouse_capture,
                mouse_look,
                player_movement,
                update_grounded,
            )
                .chain(),
        );
    }
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            PlayerLook::default(),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, PLAYER_LENGTH),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            Transform::from_xyz(31.0, 62.0, 31.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.65, 0.0),
            ));
        });
}

fn toggle_mouse_capture(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }

    if mouse.just_pressed(MouseButton::Left) {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn mouse_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    cursor: Query<&CursorOptions, With<PrimaryWindow>>,
    mut player: Query<&mut PlayerLook, With<Player>>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(cursor) = cursor.single() else {
        return;
    };

    if cursor.grab_mode != CursorGrabMode::Locked {
        mouse_motion.clear();
        return;
    }

    let Ok(mut look) = player.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;

    for event in mouse_motion.read() {
        delta += event.delta;
    }

    // Turn left/right and look up/down.
    look.yaw -= delta.x * MOUSE_SENSITIVITY;
    look.pitch = (look.pitch - delta.y * MOUSE_SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH);

    // Apply BOTH yaw and pitch to the camera.
    // The player body stays rotation-locked for physics stability.
    if let Ok(mut camera_transform) = cameras.single_mut() {
        camera_transform.rotation =
            Quat::from_rotation_y(look.yaw) * Quat::from_rotation_x(look.pitch)
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut players: Query<(Entity, &Transform, &PlayerLook, &mut LinearVelocity), With<Player>>,
    spatial_query: SpatialQuery,
) {
    let Ok((entity, transform, look, mut velocity)) = players.single_mut() else {
        return;
    };

    let forward = Vec3::new(-look.yaw.sin(), 0.0, -look.yaw.cos());
    let right = Vec3::new(look.yaw.cos(), 0.0, -look.yaw.sin());

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    // Set horizontal speed without overwriting vertical velocity.
    velocity.x = direction.x * PLAYER_SPEED;
    velocity.z = direction.z * PLAYER_SPEED;

    if keyboard.just_pressed(KeyCode::Space) {
        let shape = Collider::capsule(PLAYER_RADIUS * 0.95, PLAYER_LENGTH * 0.95);
        let grounded = spatial_query
            .cast_shape(
                &shape,
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig::from_max_distance(0.15),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            )
            .is_some_and(|hit| (transform.rotation * hit.normal1).dot(Vec3::Y) > 0.65);

        if grounded {
            velocity.y = JUMP_SPEED;
        }
    }
}

fn update_grounded(
    mut commands: Commands,
    players: Query<(Entity, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    for (entity, transform) in &players {
        let shape = Collider::capsule(PLAYER_RADIUS * 0.95, PLAYER_LENGTH * 0.95);

        let grounded = spatial_query
            .cast_shape(
                &shape,
                transform.translation,
                transform.rotation,
                Dir3::NEG_Y,
                &ShapeCastConfig::from_max_distance(0.15),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            )
            .is_some_and(|hit| (transform.rotation * hit.normal1).dot(Vec3::Y) > 0.65);

        if grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

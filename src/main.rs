mod meshing;
mod terrain;

mod player;
use player::PlayerPlugin;

use avian3d::prelude::*;
use bevy::prelude::*;

use meshing::build_chunk_mesh;
use terrain::TerrainChunk;

const WORLD_SEED: u32 = 42;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk = TerrainChunk::generate(WORLD_SEED, 0, 0);
    let chunk_mesh = build_chunk_mesh(&chunk);

    let mesh_handle = meshes.add(chunk_mesh.mesh);

    // Build a static terrain collider from the same triangle geometry.
    let collider = Collider::trimesh(chunk_mesh.vertices, chunk_mesh.triangles);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Static,
        collider,
        Transform::IDENTITY,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
}

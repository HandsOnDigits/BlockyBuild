mod meshing;
mod terrain;

mod block_interaction;
use block_interaction::{BlockInteractionPlugin, EditableTerrain};

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
        .add_plugins(BlockInteractionPlugin)
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

    let terrain_entity = commands
        .spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 1.0,
                ..default()
            })),
            RigidBody::Static,
            Collider::trimesh(chunk_mesh.vertices, chunk_mesh.triangles),
            Transform::IDENTITY,
        ))
        .id();

    commands.insert_resource(EditableTerrain {
        chunk,
        entity: terrain_entity,
        mesh: mesh_handle,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
}

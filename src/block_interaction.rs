use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    meshing::build_chunk_mesh,
    player::Player,
    terrain::{AIR, CHUNK_SIZE, DIRT, TerrainChunk},
};

const REACH: f32 = 7.0;
const PLAYER_HALF_WIDTH: f32 = 0.35;
const PLAYER_HALF_HEIGHT: f32 = 0.9;

#[derive(Resource)]
pub struct EditableTerrain {
    pub chunk: TerrainChunk,
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
}

pub struct BlockInteractionPlugin;

impl Plugin for BlockInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, block_interaction);
    }
}

fn block_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    players: Query<&Transform, With<Player>>,
    mut terrain: ResMut<EditableTerrain>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let breaking = buttons.just_pressed(MouseButton::Left);
    let placing = buttons.just_pressed(MouseButton::Right);

    if !breaking && !placing {
        return;
    }

    let Some(camera) = cameras.iter().next() else {
        return;
    };

    let origin = camera.translation();
    let direction = camera.forward().as_vec3();

    let Some((target, adjacent)) = raycast_voxels(&terrain.chunk, origin, direction, REACH) else {
        return;
    };

    if breaking {
        set_block(&mut terrain.chunk, target, AIR);
    } else if placing {
        let Some(player) = players.iter().next() else {
            return;
        };

        if !in_bounds(adjacent) || block_at(&terrain.chunk, adjacent) != AIR {
            return;
        }

        if overlaps_player(adjacent, player.translation) {
            return;
        }

        set_block(&mut terrain.chunk, adjacent, DIRT);
    }

    // Rebuild both the visual mesh and the physics collider.
    let chunk_mesh = build_chunk_mesh(&terrain.chunk);
    let collider = Collider::trimesh(chunk_mesh.vertices, chunk_mesh.triangles);

    // Replace the mesh asset and remove the old one to avoid
    // accumulating unused mesh assets during repeated edits.
    let old_mesh = terrain.mesh.clone();
    let new_mesh = meshes.add(chunk_mesh.mesh);
    meshes.remove(old_mesh.id());

    terrain.mesh = new_mesh.clone();

    commands
        .entity(terrain.entity)
        .insert((Mesh3d(new_mesh), collider));
}

fn raycast_voxels(
    chunk: &TerrainChunk,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<(IVec3, IVec3)> {
    let direction = direction.normalize_or_zero();

    if direction == Vec3::ZERO {
        return None;
    }

    let mut voxel = origin.floor().as_ivec3();

    // The player should normally be in air. Don't select a block
    // containing the camera itself.
    if block_at(chunk, voxel) != AIR {
        return None;
    }

    let step = IVec3::new(
        direction.x.total_cmp(&0.0) as i32,
        direction.y.total_cmp(&0.0) as i32,
        direction.z.total_cmp(&0.0) as i32,
    );

    // Convert comparison results into -1, 0, or 1.
    let step = IVec3::new(sign(direction.x), sign(direction.y), sign(direction.z));

    let delta = Vec3::new(
        reciprocal_abs(direction.x),
        reciprocal_abs(direction.y),
        reciprocal_abs(direction.z),
    );

    let mut next = Vec3::new(
        next_boundary(origin.x, voxel.x, direction.x),
        next_boundary(origin.y, voxel.y, direction.y),
        next_boundary(origin.z, voxel.z, direction.z),
    );

    let mut distance = 0.0;
    let mut previous = voxel;

    while distance <= max_distance {
        let axis = if next.x <= next.y && next.x <= next.z {
            0
        } else if next.y <= next.z {
            1
        } else {
            2
        };

        distance = next[axis];
        if distance > max_distance {
            break;
        }

        previous = voxel;
        voxel[axis] += step[axis];
        next[axis] += delta[axis];

        if block_at(chunk, voxel) != AIR {
            return Some((voxel, previous));
        }
    }

    None
}

fn sign(value: f32) -> i32 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

fn reciprocal_abs(value: f32) -> f32 {
    if value.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        value.abs().recip()
    }
}

fn next_boundary(origin: f32, voxel: i32, direction: f32) -> f32 {
    if direction > 0.0 {
        (voxel as f32 + 1.0 - origin) / direction
    } else if direction < 0.0 {
        (origin - voxel as f32) / -direction
    } else {
        f32::INFINITY
    }
}

fn in_bounds(pos: IVec3) -> bool {
    let size = CHUNK_SIZE as i32;

    pos.x >= 0 && pos.x < size && pos.y >= 0 && pos.y < size && pos.z >= 0 && pos.z < size
}

fn block_at(chunk: &TerrainChunk, pos: IVec3) -> u16 {
    if !in_bounds(pos) {
        return AIR;
    }

    let index = binary_greedy_meshing::pad_linearize::<CHUNK_SIZE>(
        pos.x as usize,
        pos.y as usize,
        pos.z as usize,
    );

    chunk.voxels[index]
}

fn set_block(chunk: &mut TerrainChunk, pos: IVec3, block: u16) {
    if !in_bounds(pos) {
        return;
    }

    let index = binary_greedy_meshing::pad_linearize::<CHUNK_SIZE>(
        pos.x as usize,
        pos.y as usize,
        pos.z as usize,
    );

    chunk.voxels[index] = block;
}

fn overlaps_player(block: IVec3, player: Vec3) -> bool {
    let min = block.as_vec3();
    let max = min + Vec3::ONE;

    let player_half = Vec3::new(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT, PLAYER_HALF_WIDTH);

    let player_min = player - player_half;
    let player_max = player + player_half;

    min.x < player_max.x
        && max.x > player_min.x
        && min.y < player_max.y
        && max.y > player_min.y
        && min.z < player_max.z
        && max.z > player_min.z
}

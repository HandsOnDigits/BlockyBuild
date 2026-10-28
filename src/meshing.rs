use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use binary_greedy_meshing as bgm;

use crate::terrain::{CHUNK_SIZE, DIRT, GRASS, STONE, TerrainChunk};

pub struct ChunkMesh {
    pub mesh: Mesh,
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

fn block_color(id: u32) -> [f32; 4] {
    match id as u16 {
        GRASS => [0.20, 0.65, 0.16, 1.0],
        DIRT => [0.40, 0.25, 0.13, 1.0],
        STONE => [0.48, 0.49, 0.51, 1.0],
        _ => [1.0, 0.0, 1.0, 1.0],
    }
}

/// Convert the packed quads into a standard Bevy mesh.
///
/// Face order follows binary-greedy-meshing:
/// Up, Down, Right, Left, Front, Back.
pub fn build_chunk_mesh(chunk: &TerrainChunk) -> ChunkMesh {
    let mut mesher = bgm::Mesher::<CHUNK_SIZE>::new();
    let transparent = BTreeSet::new();
    mesher.mesh(&chunk.voxels, &transparent);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let faces = [
        (Vec3::Y, 0_u32),
        (-Vec3::Y, 1),
        (Vec3::X, 2),
        (-Vec3::X, 3),
        (Vec3::Z, 4),
        (-Vec3::Z, 5),
    ];

    for (face_index, quads) in mesher.quads.iter().enumerate() {
        let (normal, _) = faces[face_index];

        for &quad in quads {
            let quad = quad.0;

            let block_id = (quad >> 32) as u32;
            let h = ((quad >> 24) & 0xff) as f32;
            let w = ((quad >> 18) & 0x3f) as f32;
            let x = (quad & 0x3f) as f32;
            let y = ((quad >> 6) & 0x3f) as f32;
            let z = ((quad >> 12) & 0x3f) as f32;

            // The packed quad origin and its width/height depend on face
            // orientation. This helper expands it to four local vertices.
            let p = face_vertices(face_index, x, y, z, w, h);

            let base = positions.len() as u32;
            for (i, vertex) in p.iter().enumerate() {
                positions.push([vertex.x, vertex.y, vertex.z]);
                normals.push(normal.to_array());
                colors.push(block_color(block_id));

                // Repeat the UVs across the merged face.
                let uv = match i {
                    0 => [0.0, 0.0],
                    1 => [w, 0.0],
                    2 => [0.0, h],
                    _ => [w, h],
                };
                uvs.push(uv);
            }

            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
    }

    let triangles = indices
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    let vertices = positions.iter().map(|p| Vec3::from_array(*p)).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    ChunkMesh {
        mesh,
        vertices,
        triangles,
    }
}

fn face_vertices(face: usize, x: f32, y: f32, z: f32, w: f32, h: f32) -> [Vec3; 4] {
    // These orientations correspond to the packed quad conventions.
    match face {
        0 => [
            Vec3::new(x + w, y, z + h),
            Vec3::new(x + w, y, z),
            Vec3::new(x, y, z + h),
            Vec3::new(x, y, z),
        ],
        1 => [
            Vec3::new(x - w, y, z + h),
            Vec3::new(x - w, y, z),
            Vec3::new(x, y, z + h),
            Vec3::new(x, y, z),
        ],
        2 => [
            Vec3::new(x, y, z),
            Vec3::new(x, y, z + h),
            Vec3::new(x, y - w, z),
            Vec3::new(x, y - w, z + h),
        ],
        3 => [
            Vec3::new(x, y, z),
            Vec3::new(x, y, z + h),
            Vec3::new(x, y + w, z),
            Vec3::new(x, y + w, z + h),
        ],
        4 => [
            Vec3::new(x - w, y + h, z),
            Vec3::new(x - w, y, z),
            Vec3::new(x, y + h, z),
            Vec3::new(x, y, z),
        ],
        _ => [
            Vec3::new(x, y, z),
            Vec3::new(x, y + h, z),
            Vec3::new(x + w, y, z),
            Vec3::new(x + w, y + h, z),
        ],
    }
}

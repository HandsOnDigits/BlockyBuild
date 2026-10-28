use binary_greedy_meshing as bgm;
use noise::{NoiseFn, Perlin};

pub const CHUNK_SIZE: usize = 62;

pub const AIR: u16 = 0;
pub const GRASS: u16 = 1;
pub const DIRT: u16 = 2;
pub const STONE: u16 = 3;

pub struct TerrainChunk {
    /// 64³ voxels including the mesher's one-voxel border.
    pub voxels: Vec<u16>,
}

impl TerrainChunk {
    pub fn generate(seed: u32, chunk_x: i32, chunk_z: i32) -> Self {
        let mut voxels = vec![AIR; bgm::Mesher::<CHUNK_SIZE>::CS_P3];

        let continental = Perlin::new(seed);
        let detail = Perlin::new(seed.wrapping_add(1));
        let ridge = Perlin::new(seed.wrapping_add(2));

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_x = chunk_x * CHUNK_SIZE as i32 + x as i32;
                let world_z = chunk_z * CHUNK_SIZE as i32 + z as i32;

                let fx = world_x as f64;
                let fz = world_z as f64;

                let broad = continental.get([fx * 0.009, fz * 0.009]);
                let fine = detail.get([fx * 0.035, fz * 0.035]);

                // Ridge noise makes sharper mountain ranges.
                let r = ridge.get([fx * 0.016, fz * 0.016]);
                let ridged = 1.0 - r.abs();

                let height = (19.0 + broad * 13.0 + ridged * 19.0 + fine * 3.0)
                    .round()
                    .clamp(3.0, 58.0) as usize;

                for y in 0..=height {
                    let block = if y == height {
                        GRASS
                    } else if y + 3 >= height {
                        DIRT
                    } else {
                        STONE
                    };

                    let index = bgm::pad_linearize::<CHUNK_SIZE>(x, y, z);
                    voxels[index] = block;
                }
            }
        }

        Self { voxels }
    }
}

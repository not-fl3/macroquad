use crate::aabb::AABB;
use crate::{CHUNK_SIZE, PIXELS_PER_TEXTURE, VOXEL_HALF};
use macroquad::{models::Vertex as Vert, prelude::*};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct Block {
    // Index into World.block_types. IDs are assigned in order of registration
    // starting from 0
    pub typ: u16,
}

pub struct Chunk {
    pub position: IVec3,
    /// Position can contain negative numbers.
    pub modified: bool,
    pub data: [Option<Block>; CHUNK_SIZE.pow(3)],
    pub mesh_opaque: Mesh,
    pub mesh_transparent: Mesh,
}

pub struct World {
    // It's the game's responsability to keep track of registered block types
    pub block_types: Vec<BlockType>,
    pub chunks: HashMap<IVec3, Chunk>,
    pub texture_cache: HashMap<String, Image>,
}

pub struct BlockType {
    pub texture: String,
    pub opaque: bool,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            block_types: Vec::new(),
            texture_cache: HashMap::new(),
        }
    }

    pub async fn register(&mut self, block_type: BlockType) -> u16 {
        self.texture_cache.insert(
            block_type.texture.clone(),
            load_image(&block_type.texture).await.unwrap(),
        );
        self.block_types.push(block_type);
        (self.block_types.len() - 1) as u16
    }

    /// Removes a block but doesn't rebuild the chunk
    pub fn queue_remove_block(&mut self, position: IVec3) {
        self.load_chunk(position);

        if let Some(chunk_position) = self.chunk_contains_point(position) {
            // Already checked that it exists
            let mut chunk = self.chunks.get_mut(&chunk_position).unwrap();
            let block_at = position - chunk.position;
            let block_at_index = chunk.get_index_from_local_position(block_at);

            chunk.data[block_at_index] = None;
            chunk.modified = true;

            let neighbors = [
                chunk_position + ivec3(CHUNK_SIZE as i32, 0, 0),
                chunk_position + ivec3(-(CHUNK_SIZE as i32), 0, 0),
                chunk_position + ivec3(0, CHUNK_SIZE as i32, 0),
                chunk_position + ivec3(0, -(CHUNK_SIZE as i32), 0),
                chunk_position + ivec3(0, 0, CHUNK_SIZE as i32),
                chunk_position + ivec3(0, 0, -(CHUNK_SIZE as i32)),
            ];

            for neighbor in neighbors.iter() {
                if let Some(neighbor_chunk) = self.chunks.get_mut(&neighbor) {
                    neighbor_chunk.modified = true;
                }
            }
        }
    }

    /// Not really designed to be used other than in debugging circumstances.
    pub fn queue_rebuild_all(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.modified = true;
        }
    }

    pub fn queue_place_block_if_not_already_there(&mut self, position: IVec3, block: Block) {
        let mut place_block = true;

        if let Some(other_block) = self.get_block_at(position) {
            if other_block.typ == block.typ {
                place_block = false;
            }
        }

        if place_block {
            self.queue_place_block(position, block);
        }
    }

    /// Place a block into the block data for a chunk, but wait to rebuild the
    /// actual geometry by marking it as dirty.
    pub fn queue_place_block(&mut self, position: IVec3, block: Block) {
        self.load_chunk(position);

        if let Some(chunk_position) = self.chunk_contains_point(position) {
            // Already checked that it exists
            let mut chunk = self.chunks.get_mut(&chunk_position).unwrap();
            let block_at = position - chunk.position;
            let block_at_index = chunk.get_index_from_local_position(block_at);

            chunk.data[block_at_index] = Some(block);
            chunk.modified = true;
        }
    }

    pub fn get_block_at(&self, position: IVec3) -> Option<Block> {
        if let Some(chunk_position) = self.chunk_contains_point(position) {
            let block_chunk_offset = position - chunk_position;
            // Already checked that it exists
            let chunk = self.chunks.get(&chunk_position).unwrap();
            let block_index = chunk.get_index_from_local_position(block_chunk_offset);
            return chunk.data[block_index];
        }

        None
    }

    pub fn rebuild_all(&mut self) {
        // Build meshes/textures OUTSIDE of the chunk using multiple READ refs
        let rebuild_chunk = |(_chunk_index, chunk): (&IVec3, &Chunk)| {
            if chunk.modified {
                Some(chunk.rebuild(&self.chunks, &self.block_types, &self.texture_cache))
            } else {
                None
            }
        };

        let new_meshes: Vec<Option<(IVec3, Mesh, Mesh, Image)>> =
            self.chunks.iter().map(rebuild_chunk).collect();

        // Assign the meshes/textures ONE AT A TIME after using one WRITE ref
        for chunk_update in new_meshes {
            if let Some((chunk_index, mut new_mesh, mut new_mesh_transparent, image)) = chunk_update
            {
                // Upload the mega texture to the GPU (on the main thread)
                let mega_texture = Texture2D::from_image(&image);
                mega_texture.set_filter(FilterMode::Nearest);
                new_mesh.texture = Some(mega_texture);
                new_mesh_transparent.texture = Some(mega_texture);

                let chunk = self.chunks.get_mut(&chunk_index).unwrap();
                chunk.mesh_opaque = new_mesh;
                chunk.mesh_transparent = new_mesh_transparent;
                chunk.modified = false;
            }
        }
    }

    /// Magically loads a chunk out of thin air. Do this from disk when wanted.
    pub fn load_chunk(&mut self, position: IVec3) {
        let chunk_global_position = self.get_position_in_chunk_space(position.as_f32());

        if !self.chunks.contains_key(&chunk_global_position) {
            self.chunks
                .insert(chunk_global_position, Chunk::new(chunk_global_position));
        }
    }

    pub fn get_position_in_chunk_space(&self, position: Vec3) -> IVec3 {
        ((position / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32).as_i32()
    }

    pub fn chunk_contains_point(&self, position: IVec3) -> Option<IVec3> {
        let chunk_space = self.get_position_in_chunk_space(position.as_f32());

        if self.chunks.contains_key(&chunk_space) {
            Some(chunk_space)
        } else {
            None
        }
    }

    /// Calling AABB::get_center() will yield the block's position.
    pub fn get_collidable_blocks(&self, aabb: AABB) -> Vec<(AABB, Block)> {
        let mut aabbs = Vec::new();

        let x_min = aabb.min.x.min(aabb.max.x).round() as i32;
        let y_min = aabb.min.y.min(aabb.max.y).round() as i32;
        let z_min = aabb.min.z.min(aabb.max.z).round() as i32;
        let x_max = aabb.max.x.max(aabb.min.x).round() as i32;
        let y_max = aabb.max.y.max(aabb.min.y).round() as i32;
        let z_max = aabb.max.z.max(aabb.min.z).round() as i32;

        for x in x_min..=x_max {
            for y in y_min..=y_max {
                for z in z_min..=z_max {
                    let block_position = ivec3(x, y, z);

                    if let Some(block) = self.get_block_at(block_position) {
                        aabbs.push((
                            AABB::from_box(block_position.as_f32(), Vec3::ONE * VOXEL_HALF),
                            block,
                        ))
                    }
                }
            }
        }

        aabbs
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    Front,
    Back,
    Top,
    Bottom,
    Right,
    Left,
}
impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::Front,
            Direction::Back,
            Direction::Top,
            Direction::Bottom,
            Direction::Right,
            Direction::Left,
        ]
        .iter()
        .cloned()
    }
}

impl Into<IVec3> for Direction {
    fn into(self) -> IVec3 {
        match self {
            Self::Right => ivec3(1, 0, 0),
            Self::Left => ivec3(-1, 0, 0),
            Self::Top => ivec3(0, 1, 0),
            Self::Bottom => ivec3(0, -1, 0),
            Self::Front => ivec3(0, 0, 1),
            Self::Back => ivec3(0, 0, -1),
        }
    }
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            modified: false,
            data: [None; CHUNK_SIZE.pow(3)],
            mesh_opaque: Mesh {
                vertices: Vec::with_capacity(0),
                indices: Vec::with_capacity(0),
                texture: Some(Texture2D::empty()),
            },
            mesh_transparent: Mesh {
                vertices: Vec::with_capacity(0),
                indices: Vec::with_capacity(0),
                texture: Some(Texture2D::empty()),
            },
        }
    }

    /// Since positions are just positive-only offsets from the chunk's
    /// position, they are never negative.
    pub fn get_local_position_from_index(&self, index: usize) -> IVec3 {
        ivec3(
            index as i32 % CHUNK_SIZE as i32,
            index as i32 / CHUNK_SIZE as i32 % CHUNK_SIZE as i32,
            index as i32 / (CHUNK_SIZE * CHUNK_SIZE) as i32,
        )
    }

    /// Since positions are just positive-only offsets from the chunk's
    /// position, they are never negative.
    pub fn get_index_from_local_position(&self, position: IVec3) -> usize {
        ((position.z * CHUNK_SIZE as i32 + position.y) * CHUNK_SIZE as i32 + position.x) as usize
    }

    pub fn get_position_in_chunk_space(&self, position: Vec3) -> IVec3 {
        ((position / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32).as_i32()
    }

    pub fn chunk_contains_point(
        &self,
        position: IVec3,
        chunks: &HashMap<IVec3, Chunk>,
    ) -> Option<IVec3> {
        let chunk_space = self.get_position_in_chunk_space(position.as_f32());

        if chunks.contains_key(&chunk_space) {
            Some(chunk_space)
        } else {
            None
        }
    }

    pub fn get_block_at(&self, position: IVec3, chunks: &HashMap<IVec3, Chunk>) -> Option<Block> {
        match self.chunk_contains_point(position, chunks) {
            Some(chunk_position) => {
                let block_chunk_offset = position - chunk_position;
                // Already checked that it exists
                let chunk = chunks.get(&chunk_position).unwrap();
                let block_index = chunk.get_index_from_local_position(block_chunk_offset);
                chunk.data[block_index]
            }

            None => None,
        }
    }

    fn face_vertices(&self, direction: Direction) -> [Vec3; 4] {
        match direction {
            Direction::Right => [
                vec3(VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
            ],

            Direction::Left => [
                vec3(-VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(-VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
                vec3(-VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
                vec3(-VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
            ],

            Direction::Top => [
                vec3(-VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
                vec3(-VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
            ],

            Direction::Bottom => [
                vec3(-VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
                vec3(-VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
            ],

            Direction::Front => [
                vec3(-VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
                vec3(VOXEL_HALF, VOXEL_HALF, VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
                vec3(-VOXEL_HALF, -VOXEL_HALF, VOXEL_HALF),
            ],

            Direction::Back => [
                vec3(-VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, VOXEL_HALF, -VOXEL_HALF),
                vec3(VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
                vec3(-VOXEL_HALF, -VOXEL_HALF, -VOXEL_HALF),
            ],
        }
    }

    fn generate_face(
        &self,
        this_block: Block,
        this_block_type: &BlockType,
        index: usize,
        direction: Direction,
        other_chunks: &HashMap<IVec3, Chunk>,
        block_types: &Vec<BlockType>,
        uv: Vec4,
        vertices: &mut Vec<Vert>,
        indices: &mut Vec<u16>,
    ) {
        let block_chunk_offset = self.get_local_position_from_index(index);
        let global_block_position = self.position + block_chunk_offset;
        let mut generate_face = true;

        /*
        To generate a face, follow these rules:
         - If both blocks are opaque, don't generate a face
         - If both blocks are transparent, don't generate a face
         - If one block is opaque and another is transparent, generate a face
         - If there is no neighbor, generate a face
        */

        // Check accross chunk boundaries to ensure that the least amount of
        // faces are being generated
        if let Some(other_block) =
            self.get_block_at(global_block_position + direction.into(), other_chunks)
        {
            let other_block_type = &block_types[other_block.typ as usize];

            let same_type = this_block.typ == other_block.typ;
            let same_opaque = this_block_type.opaque == other_block_type.opaque;

            // Water + Water, Stone + Stone
            if same_type && same_opaque {
                generate_face = false;
            }
            // Stone + Dirt, Water + Lava
            else if !same_type && same_opaque {
                generate_face = !this_block_type.opaque;
            }
            // Stone + Water
            else if !same_type && !same_opaque {
                // Generate face only for opaque blocks that are adjacent to
                // water, not the water itself
                generate_face = this_block_type.opaque;
            }
        }

        if generate_face {
            let face = self.face_vertices(direction);
            let base_indices = [0u16, 1u16, 2u16, 0u16, 2u16, 3u16];

            indices.extend(base_indices.iter().map(|i| i + vertices.len() as u16));

            let global_block_position = self.position.as_f32() + block_chunk_offset.as_f32();
            let color = WHITE;

            vertices.push(Vert {
                position: global_block_position + face[0],
                uv: uv.xy(),
                color: color,
            });

            vertices.push(Vert {
                position: global_block_position + face[1],
                uv: uv.zy(),
                color: color,
            });

            vertices.push(Vert {
                position: global_block_position + face[2],
                uv: uv.zw(),
                color: color,
            });

            vertices.push(Vert {
                position: global_block_position + face[3],
                uv: uv.xw(),
                color: color,
            });
        }
    }

    pub fn rebuild(
        &self,
        other_chunks: &HashMap<IVec3, Chunk>,
        block_types: &Vec<BlockType>,
        texture_cache: &HashMap<String, Image>,
    ) -> (IVec3, Mesh, Mesh, Image) {
        // Get number of unique textures so megatexture size can be determined
        let mut unique_textures = HashSet::new();

        // Needed for vertex/index Vec capacity planning
        let mut num_blocks = 0;

        for block_cell in self.data.iter() {
            if let Some(block) = block_cell {
                unique_textures.insert(block.typ);
                num_blocks += 1;
            }
        }

        let mega_texture_num_tiles_xy = (unique_textures.len() as f32).sqrt().ceil();

        // Build a new mega texture
        let width_and_height = mega_texture_num_tiles_xy as u16 * PIXELS_PER_TEXTURE as u16;

        let mut mega = Image::gen_image_color(
            width_and_height,
            width_and_height,
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        let (mut col, mut row) = (0u32, 0u32);
        let mut tiles = HashMap::<u16, Vec4>::new();

        // Draw each unique texture onto the mega texture and store its UVs
        for block_type_id in unique_textures {
            // Textures are loaded when block types are registered, so these
            // are guaranteed to succeed
            let block_type = &block_types[block_type_id as usize];
            let texture = texture_cache.get(&block_type.texture).unwrap();

            draw_image(
                &mut mega,
                &texture,
                col * PIXELS_PER_TEXTURE as u32,
                row * PIXELS_PER_TEXTURE as u32,
            );

            // Store the tile for UV generation later
            // Square, width & height are equal
            let size = PIXELS_PER_TEXTURE as f32 / mega.width() as f32;
            let top_left = vec2(col as f32 * size, row as f32 * size);
            let lower_right = top_left + vec2(size, size);

            tiles.insert(
                block_type_id,
                vec4(top_left.x, top_left.y, lower_right.x, lower_right.y),
            );

            col += 1u32;
            if col >= mega_texture_num_tiles_xy as u32 {
                col = 0u32;
                // No need to check col as all images are guaranteed to fit in
                // chosen squre size
                row += 1u32;
            }
        }

        // Generate mesh using generated mega texture and UVs
        let mut vertices = Vec::with_capacity(num_blocks * 4);
        // Triangle count * num sides
        let mut indices = Vec::with_capacity(num_blocks * 6 * 6);
        let mut vertices_transparent = Vec::with_capacity(num_blocks * 4);
        // Triangle count * num sides
        let mut indices_transparent = Vec::with_capacity(num_blocks * 6 * 6);

        for (index, block_slot) in self.data.iter().enumerate() {
            if let Some(block) = block_slot {
                let block_type = &block_types[block.typ as usize];
                let uv = tiles[&block.typ];

                let (verts, inds) = if block_type.opaque {
                    (&mut vertices, &mut indices)
                } else {
                    (&mut vertices_transparent, &mut indices_transparent)
                };

                // Check all 6 sides for neighbors that would obscure faces
                // that face them
                for direction in Direction::iter() {
                    self.generate_face(
                        *block,
                        block_type,
                        index,
                        direction,
                        other_chunks,
                        block_types,
                        uv,
                        verts,
                        inds,
                    );
                }
            }
        }

        // Return the newly created chunk!
        (
            self.position,
            Mesh {
                vertices,
                indices,
                texture: None,
            },
            Mesh {
                vertices: vertices_transparent,
                indices: indices_transparent,
                texture: None,
            },
            mega,
        )
    }
}

fn draw_image(destination: &mut Image, source: &Image, at_x: u32, at_y: u32) {
    assert!(destination.width() as u32 >= source.width() as u32 + at_x);
    assert!(destination.height() as u32 >= source.height() as u32 + at_y);

    for y in 0u32..source.height() as u32 {
        for x in 0u32..source.width() as u32 {
            let pixel = source.get_pixel(x, y);
            destination.set_pixel(at_x + x, at_y + y, pixel);
        }
    }
}

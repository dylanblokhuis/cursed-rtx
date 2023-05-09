use block_mesh::{
    greedy_quads,
    ndshape::{RuntimeShape, Shape},
    GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use windows::Win32::System::SystemServices::{
    D3DFVF_DIFFUSE, D3DFVF_NORMAL, D3DFVF_TEX1, D3DFVF_XYZ,
};

#[derive(Clone, Copy, Eq, PartialEq)]
struct PalleteVoxel(u8);

const EMPTY: PalleteVoxel = PalleteVoxel(255);

impl Voxel for PalleteVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for PalleteVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        PalleteVoxel(self.0)
    }
}

// A 16^3 chunk with 1-voxel boundary padding.

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Mesh {
    position: [f32; 3],
    normal: [f32; 3],
    color: [u8; 4],
    uv: [f32; 2],
}

pub const MESH_FVF_FORMAT: u32 = D3DFVF_XYZ | D3DFVF_NORMAL | D3DFVF_DIFFUSE | D3DFVF_TEX1;

pub fn gen(vox_path: &str) -> (Vec<Mesh>, Vec<u32>) {
    let vox = dot_vox::load(vox_path).unwrap();
    let model = vox.models.get(0).unwrap();
    let palette = vox.palette;
    let shape = RuntimeShape::<u32, 3>::new([model.size.x + 2, model.size.y + 2, model.size.z + 2]);

    let mut voxels = vec![EMPTY; shape.size() as usize];
    for voxel in model.voxels.iter() {
        let x = voxel.x + 1;
        let y = voxel.y + 1;
        let z = voxel.z + 1;
        let index = shape.linearize([x as u32, y as u32, z as u32]);
        voxels[index as usize] = PalleteVoxel(voxel.i);
    }

    let mut buffer = GreedyQuadsBuffer::new(shape.size() as usize);
    let quads_config = RIGHT_HANDED_Y_UP_CONFIG;

    greedy_quads(
        &voxels,
        &shape,
        [0; 3],
        [model.size.x, model.size.y, model.size.z],
        &quads_config.faces,
        &mut buffer,
    );

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    let mut colors = Vec::with_capacity(num_vertices);
    let v_flip_face = false;

    for (group, face) in buffer.quads.groups.iter().zip(quads_config.faces.as_ref()) {
        for quad in group.iter() {
            let palette_index = voxels[shape.linearize(quad.minimum) as usize].0;
            colors.extend_from_slice(&[palette[palette_index as usize]; 4]);
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(
                &face
                    .quad_mesh_positions(quad, 1.0)
                    .map(|position| position.map(|x| x - 1.0)), // corrects the 1 offset introduced by the meshing.
            );
            uvs.extend_from_slice(&face.tex_coords(quads_config.u_flip_face, v_flip_face, quad));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    println!("num_vertices: {}", num_vertices);

    assert!(positions.len() == normals.len());
    assert!(!positions.is_empty());

    let mut meshes = Vec::new();
    for i in 0..positions.len() {
        let mesh = Mesh {
            position: [positions[i][0], positions[i][1], positions[i][2]],
            normal: [normals[i][0], normals[i][1], normals[i][2]],
            color: [colors[i].b, colors[i].g, colors[i].r, colors[i].a],
            uv: [uvs[i][0], uvs[i][1]],
        };
        meshes.push(mesh);
    }
    (meshes, indices)
}

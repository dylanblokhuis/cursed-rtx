use std::{
    mem::{size_of, size_of_val},
    ptr::null_mut,
};

use glam::{Mat4, Quat, Vec3};
use windows::Win32::Graphics::Direct3D9::{
    IDirect3DDevice9, IDirect3DIndexBuffer9, IDirect3DVertexBuffer9, D3DFMT_INDEX32,
    D3DPOOL_DEFAULT, D3DUSAGE_DYNAMIC, D3DUSAGE_WRITEONLY,
};

use crate::{
    glam_to_wmatrix,
    mesh::{Mesh, MESH_FVF_FORMAT},
    DrawCmd,
};

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

pub struct Model {
    meshes: Vec<Mesh>,
    indices: Vec<u32>,
    pub transform: Transform,
    pub fvf: u32,
}

impl Model {
    pub fn new(data: (Vec<Mesh>, Vec<u32>), transform: Transform) -> Self {
        Model {
            meshes: data.0,
            indices: data.1,
            transform,
            fvf: MESH_FVF_FORMAT,
        }
    }

    pub fn stride(&self) -> u32 {
        size_of::<Mesh>() as u32
    }

    pub fn num_vertices(&self) -> u32 {
        self.meshes.len() as u32
    }

    pub fn primitive_count(&self) -> u32 {
        self.indices.len() as u32 / 3
    }

    pub unsafe fn create_vertex_buffer(
        &self,
        device: &IDirect3DDevice9,
    ) -> Option<IDirect3DVertexBuffer9> {
        let mut vertex_buffer: Option<IDirect3DVertexBuffer9> = None;
        device
            .CreateVertexBuffer(
                (std::mem::size_of::<Mesh>() * self.meshes.len()) as u32,
                (D3DUSAGE_DYNAMIC | D3DUSAGE_WRITEONLY) as u32,
                MESH_FVF_FORMAT,
                D3DPOOL_DEFAULT,
                &mut vertex_buffer,
                null_mut(),
            )
            .unwrap();

        let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        vertex_buffer
            .as_ref()
            .unwrap()
            .Lock(0, size_of_val(&self.meshes) as u32, &mut data_ptr, 0)
            .unwrap();

        let data_slice = std::slice::from_raw_parts_mut(data_ptr as *mut Mesh, self.meshes.len());
        data_slice.copy_from_slice(&self.meshes);
        vertex_buffer.as_ref().unwrap().Unlock().unwrap();
        vertex_buffer
    }

    pub unsafe fn create_index_buffer(
        &self,
        device: &IDirect3DDevice9,
    ) -> Option<IDirect3DIndexBuffer9> {
        let mut index_buffer: Option<IDirect3DIndexBuffer9> = None;

        device
            .CreateIndexBuffer(
                (self.indices.len() * size_of::<u32>()) as u32,
                (D3DUSAGE_DYNAMIC | D3DUSAGE_WRITEONLY) as u32,
                D3DFMT_INDEX32,
                D3DPOOL_DEFAULT,
                &mut index_buffer,
                null_mut(),
            )
            .unwrap();

        let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        index_buffer
            .as_ref()
            .unwrap()
            .Lock(0, size_of_val(&self.indices) as u32, &mut data_ptr, 0)
            .unwrap();

        let data_slice = std::slice::from_raw_parts_mut(data_ptr as *mut u32, self.indices.len());
        data_slice.copy_from_slice(&self.indices);
        index_buffer.as_ref().unwrap().Unlock().unwrap();
        index_buffer
    }

    pub unsafe fn to_draw_cmd(&self, device: &IDirect3DDevice9) -> DrawCmd {
        DrawCmd {
            vertex_buffer: unsafe { self.create_vertex_buffer(&device) },
            index_buffer: unsafe { self.create_index_buffer(&device) },
            fvf: self.fvf,
            num_vertices: self.num_vertices(),
            primitive_count: self.primitive_count(),
            vertex_stride: self.stride(),
            world_matrix: glam_to_wmatrix(self.transform.model_matrix()),
        }
    }
}

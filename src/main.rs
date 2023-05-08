use windows::Foundation::Numerics::Matrix4x4;
use windows::Win32::Graphics::Direct3D::D3DVECTOR;
use windows::Win32::System::SystemServices::{
    D3DCLEAR_TARGET, D3DCLEAR_ZBUFFER, D3DFVF_DIFFUSE, D3DFVF_XYZ,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
};

use windows::Win32::Graphics::Direct3D9::*;

use std::mem::size_of_val;
use std::ptr::null_mut;

const WINDOW_WIDTH: i32 = 640;
const WINDOW_HEIGHT: i32 = 480;

fn glam_to_wmatrix(mat: glam::Mat4) -> Matrix4x4 {
    Matrix4x4 {
        M11: mat.x_axis.x,
        M12: mat.x_axis.y,
        M13: mat.x_axis.z,
        M14: mat.x_axis.w,
        M21: mat.y_axis.x,
        M22: mat.y_axis.y,
        M23: mat.y_axis.z,
        M24: mat.y_axis.w,
        M31: mat.z_axis.x,
        M32: mat.z_axis.y,
        M33: mat.z_axis.z,
        M34: mat.z_axis.w,
        M41: mat.w_axis.x,
        M42: mat.w_axis.y,
        M43: mat.w_axis.z,
        M44: mat.w_axis.w,
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {
            return DefWindowProcA(hwnd, msg, wparam, lparam);
        }
    }
    LRESULT(0)
}

unsafe fn setup_dx_context(hwnd: HWND) -> (IDirect3D9, IDirect3DDevice9) {
    let d9_option = Direct3DCreate9(D3D_SDK_VERSION);
    match d9_option {
        Some(d9) => {
            let mut present_params = D3DPRESENT_PARAMETERS {
                BackBufferWidth: WINDOW_WIDTH as _,
                BackBufferHeight: WINDOW_HEIGHT as _,
                BackBufferFormat: D3DFMT_R5G6B5,
                BackBufferCount: 1,
                MultiSampleType: D3DMULTISAMPLE_NONE,
                MultiSampleQuality: 0,
                SwapEffect: D3DSWAPEFFECT_DISCARD,
                hDeviceWindow: hwnd,
                Windowed: BOOL(1),
                EnableAutoDepthStencil: BOOL(1),
                AutoDepthStencilFormat: D3DFMT_D24S8,
                Flags: 0,
                FullScreen_RefreshRateInHz: D3DPRESENT_RATE_DEFAULT,
                PresentationInterval: D3DPRESENT_INTERVAL_IMMEDIATE as u32,
            };
            let mut device: Option<IDirect3DDevice9> = None;
            match d9.CreateDevice(
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_HAL,
                hwnd,
                D3DCREATE_SOFTWARE_VERTEXPROCESSING as u32,
                &mut present_params,
                &mut device,
            ) {
                Ok(_) => (d9, device.unwrap()),
                _ => panic!("CreateDevice failed"),
            }
        }
        None => panic!("Direct3DCreate9 failed"),
    }
}

fn main() {
    let hinstance = unsafe { GetModuleHandleA(None).unwrap() };
    let wnd_class = WNDCLASSA {
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: HICON(0),
        hCursor: HCURSOR(0),
        hbrBackground: Default::default(),
        lpszMenuName: PCSTR(null_mut()),
        lpszClassName: PCSTR("MyClass\0".as_ptr()),
    };
    unsafe { RegisterClassA(&wnd_class) };
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: WINDOW_WIDTH as _,
        bottom: WINDOW_HEIGHT as _,
    };
    unsafe {
        AdjustWindowRect(
            &mut rect,
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            BOOL::from(false),
        )
    };
    let handle = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE(0),
            PCSTR("MyClass\0".as_ptr()),
            PCSTR("MiniWIN\0".as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            // size and position
            100,
            100,
            rect.right - rect.left,
            rect.bottom - rect.top,
            HWND(0),
            HMENU(0),
            hinstance,
            None,
        )
    };

    let (_, device) = unsafe { setup_dx_context(handle) };

    let view_matrix = glam::Mat4::look_at_lh(
        glam::Vec3::new(0.0, 0.0, -5.0),
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(0.0, 1.0, 0.0),
    );
    let proj_matrix = glam::Mat4::perspective_lh(
        60_f32.to_radians(),
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        1.0,
        1000.0,
    );
    let surface = unsafe { device.GetBackBuffer(0, 0, D3DBACKBUFFER_TYPE_MONO).unwrap() };

    // setup fixed function pipeline
    unsafe {
        device.SetRenderTarget(0, &surface).unwrap();
        device
            .SetViewport(&D3DVIEWPORT9 {
                X: 0,
                Y: 0,
                Width: WINDOW_WIDTH as u32,
                Height: WINDOW_HEIGHT as u32,
                MinZ: 0.0,
                MaxZ: 1.0,
            })
            .unwrap();
        // device
        //     .SetRenderState(D3DRS_CULLMODE, D3DCULL_NONE.0)
        //     .unwrap();
        device.SetRenderState(D3DRS_LIGHTING, 0).unwrap();
        // device.SetRenderState(D3DRS_ZENABLE, 0).unwrap();
        // device.SetRenderState(D3DRS_ALPHABLENDENABLE, 1).unwrap();
        // device.SetRenderState(D3DRS_ALPHATESTENABLE, 0).unwrap();
        // device
        //     .SetRenderState(D3DRS_BLENDOP, D3DBLENDOP_ADD.0)
        //     .unwrap();
        // device
        //     .SetRenderState(D3DRS_SRCBLEND, D3DBLEND_SRCALPHA.0)
        //     .unwrap();
        // device
        //     .SetRenderState(D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA.0)
        //     .unwrap();
        // device.SetRenderState(D3DRS_SCISSORTESTENABLE, 1).unwrap();
        device
            .SetRenderState(D3DRS_SHADEMODE, D3DSHADE_GOURAUD.0 as u32)
            .unwrap();
        // device.SetRenderState(D3DRS_FOGENABLE, 0).unwrap();

        let world_matrix = glam_to_wmatrix(glam::Mat4::IDENTITY);
        device.SetTransform(D3DTS_WORLD, &world_matrix).unwrap();
        let view_matrix = glam_to_wmatrix(view_matrix);
        device.SetTransform(D3DTS_VIEW, &view_matrix).unwrap();
        let proj_matrix = glam_to_wmatrix(proj_matrix);
        device.SetTransform(D3DTS_PROJECTION, &proj_matrix).unwrap();

        let material = D3DMATERIAL9 {
            Ambient: D3DCOLORVALUE {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            ..Default::default()
        };
        device.SetMaterial(&material).unwrap();

        let light = D3DLIGHT9 {
            Type: D3DLIGHT_DIRECTIONAL,
            Diffuse: D3DCOLORVALUE {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            Direction: D3DVECTOR {
                x: 0.0,
                y: -1.0,
                z: 1.0,
            },
            ..Default::default()
        };
        device.SetLight(0, &light).unwrap();
        device.LightEnable(0, true).unwrap();
    }

    // vertex buffer
    let triangle_vertices = [
        Vertex {
            position: [0.0, 1.0, 0.0],
            color: [0, 255, 0, 1],
        },
        Vertex {
            position: [1.0, -1.0, 0.0],
            color: [0, 0, 255, 1],
        },
        Vertex {
            position: [-1.0, -1.0, 0.0],
            color: [255, 0, 0, 1],
        },
    ];

    let mut vertex_buffer: Option<IDirect3DVertexBuffer9> = None;
    unsafe {
        device
            .CreateVertexBuffer(
                (std::mem::size_of::<Vertex>() * triangle_vertices.len()) as u32,
                (D3DUSAGE_DYNAMIC | D3DUSAGE_WRITEONLY) as u32,
                D3DFVF_XYZ | D3DFVF_DIFFUSE,
                D3DPOOL_DEFAULT,
                &mut vertex_buffer,
                null_mut(),
            )
            .unwrap();

        let mut data_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        vertex_buffer
            .as_ref()
            .unwrap()
            .Lock(0, size_of_val(&triangle_vertices) as u32, &mut data_ptr, 0)
            .unwrap();

        let data_slice =
            std::slice::from_raw_parts_mut(data_ptr as *mut Vertex, triangle_vertices.len());
        data_slice.copy_from_slice(&triangle_vertices);
        vertex_buffer.as_ref().unwrap().Unlock().unwrap();
    }

    let mut msg: MSG = MSG::default();
    loop {
        unsafe {
            while PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).0 != 0 {
                if msg.message == WM_QUIT {
                    break;
                }

                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            if msg.message == WM_QUIT {
                break;
            }

            device
                .Clear(
                    0,
                    null_mut(),
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                    255, // blue
                    1.0,
                    0,
                )
                .unwrap();
            device.BeginScene().unwrap();
            device
                .SetStreamSource(
                    0,
                    vertex_buffer.as_ref().unwrap(),
                    0,
                    std::mem::size_of::<Vertex>() as u32,
                )
                .unwrap();
            device.SetFVF(D3DFVF_XYZ | D3DFVF_DIFFUSE).unwrap();
            device.DrawPrimitive(D3DPT_TRIANGLELIST, 0, 1).unwrap();
            device.EndScene().unwrap();
            device
                .Present(null_mut(), null_mut(), None, null_mut())
                .unwrap();
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [u8; 4],
}

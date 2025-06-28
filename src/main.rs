use std::{io, path::PathBuf, time::Instant};

use bgfx::*;
use bgfx_rs::bgfx;
use glam::{EulerRot, Mat4, Vec3};
use glfw::{Action, Key};

mod platform;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

#[repr(C, packed)]
struct PosColorVertex {
    _x: f32,
    _y: f32,
    _z: f32,
    _abgr: u32,
}

#[rustfmt::skip]
static CUBE_VERTICES: [PosColorVertex; 8] = [
    PosColorVertex { _x: -1.0, _y:  1.0, _z:  1.0, _abgr: 0xff000000 },
    PosColorVertex { _x:  1.0, _y:  1.0, _z:  1.0, _abgr: 0xff0000ff },
    PosColorVertex { _x: -1.0, _y: -1.0, _z:  1.0, _abgr: 0xff00ff00 },
    PosColorVertex { _x:  1.0, _y: -1.0, _z:  1.0, _abgr: 0xff00ffff },
    PosColorVertex { _x: -1.0, _y:  1.0, _z: -1.0, _abgr: 0xffff0000 },
    PosColorVertex { _x:  1.0, _y:  1.0, _z: -1.0, _abgr: 0xffff00ff },
    PosColorVertex { _x: -1.0, _y: -1.0, _z: -1.0, _abgr: 0xffffff00 },
    PosColorVertex { _x:  1.0, _y: -1.0, _z: -1.0, _abgr: 0xffffffff },
];

#[rustfmt::skip]
static CUBE_INDICES: [u16; 36] = [
    0, 1, 2, // 0
    1, 3, 2,
    4, 6, 5, // 2
    5, 6, 7,
    0, 2, 4, // 4
    4, 2, 6,
    1, 5, 3, // 6
    5, 7, 3,
    0, 4, 1, // 8
    4, 5, 1,
    2, 3, 6, // 10
    6, 3, 7,
];

fn load_shader_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut path = PathBuf::with_capacity(512);
    path.push("shaders");

    match bgfx::get_renderer_type() {
        RendererType::Direct3D11 => path.push("dx11"),
        RendererType::OpenGL => path.push("glsl"),
        RendererType::Metal => path.push("metal"),
        RendererType::OpenGLES => path.push("essl"),
        RendererType::Vulkan => path.push("spirv"),
        e => panic!("Unsupported render type {e:#?}"),
    }

    path.push(format!("{name}.bin"));

    let mut data = std::fs::read(&path).map_err(|e| {
        io::Error::other(format!("Failed to open {}: {e:?}", path.to_string_lossy()))
    })?;
    data.push(0); // this is to terminate the data
    Ok(data)
}

fn load_shader_program(vs: &str, ps: &str) -> std::io::Result<Program> {
    let vs_data = load_shader_file(vs)?;
    let ps_data = load_shader_file(ps)?;

    let vs_data = Memory::copy(&vs_data);
    let ps_data = Memory::copy(&ps_data);

    let vs_shader = bgfx::create_shader(&vs_data);
    let ps_shader = bgfx::create_shader(&ps_data);

    Ok(bgfx::create_program(&vs_shader, &ps_shader, false))
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

    let (mut window, events) = glfw
        .create_window(
            WIDTH as _,
            HEIGHT as _,
            "helloworld.rs bgfx-rs example - ESC to close",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);

    let mut init = Init::new();

    init.type_r = platform::get_render_type();
    init.resolution.width = WIDTH as u32;
    init.resolution.height = HEIGHT as u32;
    init.resolution.reset = ResetFlags::VSYNC.bits();
    init.platform_data = platform::get_platform_data(&window);

    if !bgfx::init(&init) {
        panic!("failed to init bgfx");
    }

    bgfx::set_debug(DebugFlags::TEXT.bits());
    bgfx::set_view_clear(
        0,
        ClearFlags::COLOR.bits() | ClearFlags::DEPTH.bits(),
        SetViewClearArgs {
            rgba: 0x503030ff,
            ..Default::default()
        },
    );

    let mut old_size = (0, 0);

    let layout = VertexLayoutBuilder::begin(RendererType::Noop)
        .add(Attrib::Position, 3, AttribType::Float, AddArgs::default())
        .add(
            Attrib::Color0,
            4,
            AttribType::Uint8,
            AddArgs {
                normalized: true,
                as_int: false,
            },
        )
        .end();

    let verts_mem = unsafe { Memory::reference(&CUBE_VERTICES) };
    let index_mem = unsafe { Memory::reference(&CUBE_INDICES) };

    let vbh = bgfx::create_vertex_buffer(&verts_mem, &layout, BufferFlags::NONE.bits());
    let ibh = bgfx::create_index_buffer(&index_mem, BufferFlags::NONE.bits());

    let shader_program = load_shader_program("vs_cubes", "fs_cubes").unwrap();

    let state = (StateWriteFlags::R
        | StateWriteFlags::G
        | StateWriteFlags::B
        | StateWriteFlags::A
        | StateWriteFlags::Z)
        .bits()
        | StateDepthTestFlags::LESS.bits()
        | StateCullFlags::CW.bits();

    let at = Vec3::new(0.0, 0.0, 0.0);
    let eye = Vec3::new(0.0, 0.0, -35.0);
    let up = Vec3::new(0.0, 1.0, 0.0);

    let time = Instant::now();

    let mut cursor_pos = None;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match &event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::CursorPos(x, y) => cursor_pos = Some((*x as f32, *y as f32)),
                _ => {}
            }
        }

        let t = time.elapsed().as_secs_f32();
        let size = window.get_framebuffer_size();

        if old_size != size {
            bgfx::reset(size.0 as _, size.1 as _, ResetArgs::default());
            old_size = size;
        }

        let aspect = size.0 as f32 / size.1 as f32;

        let persp = Mat4::perspective_lh(60.0 * (std::f32::consts::PI / 180.0), aspect, 0.1, 100.0);
        let view = Mat4::look_at_lh(eye, at, up);

        bgfx::set_view_rect(0, 0, 0, size.0 as _, size.1 as _);
        bgfx::touch(0);

        bgfx::set_view_transform(0, &view.to_cols_array(), &persp.to_cols_array());

        let (x_off, y_off) = cursor_pos.unwrap_or((0.0, 0.0));

        for yy in 0..11 {
            for xx in 0..11 {
                for zz in 0..11 {
                    let x = -15.0 + (xx as f32) * 3.0;
                    let y = -15.0 + (yy as f32) * 3.0;
                    let z = -15.0 + (zz as f32) * 3.0;
                    let xr = t + (xx as f32) * 0.21;
                    let yr = t + (yy as f32) * 0.37;

                    let rot = Mat4::from_euler(
                        EulerRot::XYZ,
                        xr + x_off / 1024.0,
                        yr + y_off / 1024.0,
                        0.0,
                    );
                    let transform = Mat4::from_translation(Vec3::new(x, y, z)) * rot;

                    bgfx::set_transform(&transform.to_cols_array(), 1);
                    bgfx::set_vertex_buffer(0, &vbh, 0, std::u32::MAX);
                    bgfx::set_index_buffer(&ibh, 0, std::u32::MAX);

                    bgfx::set_state(state, 0);
                    bgfx::submit(0, &shader_program, SubmitArgs::default());
                }
            }
        }

        bgfx::dbg_text_clear(DbgTextClearArgs::default());

        bgfx::dbg_text(0, 1, 0x0f, "Color can be changed with ANSI \x1b[9;me\x1b[10;ms\x1b[11;mc\x1b[12;ma\x1b[13;mp\x1b[14;me\x1b[0m code too.");
        bgfx::dbg_text(80, 1, 0x0f, "\x1b[;0m    \x1b[;1m    \x1b[; 2m    \x1b[; 3m    \x1b[; 4m    \x1b[; 5m    \x1b[; 6m    \x1b[; 7m    \x1b[0m");
        bgfx::dbg_text(80, 2, 0x0f, "\x1b[;8m    \x1b[;9m    \x1b[;10m    \x1b[;11m    \x1b[;12m    \x1b[;13m    \x1b[;14m    \x1b[;15m    \x1b[0m");
        bgfx::dbg_text(
            0,
            4,
            0x3f,
            "Description: Initialization and debug text with bgfx-rs Rust API.",
        );

        bgfx::dbg_text(0, 10, 0x3f, "I was here");

        bgfx::frame(false);
    }

    bgfx::shutdown();
}

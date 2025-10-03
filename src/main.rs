pub mod bus;
pub mod cpu;
pub mod emulator;
pub mod ppu;
pub mod rom;

use clap::Parser;
use emulator::Emulator;
use glow::HasContext;
use imgui::{Condition, Context};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;
const NES_ASPECT_RATIO: f32 = NES_WIDTH as f32 / NES_HEIGHT as f32;

const DEFAULT_WINDOW_WIDTH: u32 = NES_WIDTH * 3;
const DEFAULT_WINDOW_HEIGHT: u32 = NES_HEIGHT * 3;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    rom: String,

    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "NES Emulator",
            if args.debug {
                DEFAULT_WINDOW_WIDTH + (DEFAULT_WINDOW_WIDTH / 2) // Allow enough room for the original width + 33% for a debug window
            } else {
                DEFAULT_WINDOW_WIDTH
            },
            DEFAULT_WINDOW_HEIGHT,
        )
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    let gl = unsafe { glow::Context::from_loader_function(|s| video_subsystem.gl_get_proc_address(s) as *const _) };

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.style_mut().use_dark_colors();

    let mut imgui_platform = SdlPlatform::new(&mut imgui);
    let mut imgui_renderer = AutoRenderer::new(gl, &mut imgui).unwrap();

    let gl = imgui_renderer.gl_context();
    let nes_texture = unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            NES_WIDTH as i32,
            NES_HEIGHT as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            None,
        );

        texture
    };

    let mut debug_visible = args.debug;

    let mut event_pump = sdl_context.event_pump().unwrap();

    let rom = Rom::load(args.rom.as_str()).unwrap();
    let mut emulator = Emulator::new(rom);
    emulator.run(
        |_cpu| {
            // Per-instruction callback, which can be used for debugging/tracing
        },
        |cpu| {
            for event in event_pump.poll_iter() {
                imgui_platform.handle_event(&mut imgui, &event);

                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown {
                        keycode: Some(Keycode::F12), ..
                    } => debug_visible = !debug_visible,
                    _ => {}
                }
            }

            imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);
            let ui = imgui.frame();

            let [display_width, display_height] = ui.io().display_size;
            let emulator_width = if debug_visible { display_width * 0.67 } else { display_width };

            let mut rendered_width = 0.0;
            let mut rendered_height = 0.0;

            {
                let _padding_token = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
                ui.window("##output")
                    .position([0.0, 0.0], Condition::Always)
                    .size([emulator_width, display_height], Condition::Always)
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    .title_bar(false)
                    .menu_bar(false)
                    .scrollable(false)
                    .scroll_bar(false)
                    .draw_background(true)
                    .build(|| {
                        let content_region = ui.content_region_avail();

                        let mut display_width = content_region[0];
                        let mut display_height = display_width / NES_ASPECT_RATIO;

                        if display_height > content_region[1] {
                            display_height = content_region[1];
                            display_width = display_height * NES_ASPECT_RATIO;
                        }

                        let offset_x = (content_region[0] - display_width) * 0.5;
                        let offset_y = (content_region[1] - display_height) * 0.5;

                        let cursor_pos = ui.cursor_screen_pos();
                        ui.set_cursor_screen_pos([cursor_pos[0] + offset_x, cursor_pos[1] + offset_y]);

                        let texture_id = imgui::TextureId::new(nes_texture.0.get() as usize);
                        imgui::Image::new(texture_id, [display_width, display_height]).build(ui);

                        rendered_width = display_width;
                        rendered_height = display_height;
                    });
            }

            if debug_visible {
                let debug_width = display_width - emulator_width;

                ui.window("##debug")
                    .position([emulator_width, 0.0], Condition::Always)
                    .size([debug_width, display_height], Condition::Always)
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    .title_bar(false)
                    .build(|| {
                        if ui.collapsing_header("Display", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            ui.text(format!("Window: {}×{}", display_width, display_height));
                            ui.text(format!(
                                "Game Area: {}×{} ({:.2}x)",
                                rendered_width as u32,
                                rendered_height as u32,
                                rendered_width / NES_WIDTH as f32
                            ));
                        }

                        if ui.collapsing_header("Performance", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            ui.text(format!("FPS: {:.1}", ui.io().framerate));
                            ui.text(format!("Frame time: {:.3}ms", ui.io().delta_time * 1000.0));
                        }

                        if ui.collapsing_header("CPU Debug", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            ui.text(format!("Cycle: {}", cpu.cycles));
                            ui.text(format!("PC: ${:04X}", cpu.pc));
                            ui.text(format!("SP: ${:02X} ({})", cpu.sp, cpu.sp));
                            ui.text(format!("A: ${:02X} ({})", cpu.a, cpu.a));
                            ui.text(format!("X: ${:02X} ({})", cpu.x, cpu.x));
                            ui.text(format!("Y: ${:02X} ({})", cpu.y, cpu.y));
                            text_bitflags(ui, "STATUS", "NV-BDIZC", cpu.status.bits());
                        }

                        if ui.collapsing_header("PPU Debug", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            let ppu = &cpu.bus.borrow_mut().ppu;
                            ui.text(format!("Cycle: {}", ppu.cycle));
                            ui.text(format!("Scanline: {}", ppu.scanline));
                            ui.text(format!("Frame: {}", ppu.frame));

                            ui.separator();

                            text_bitflags(ui, "CTRL", "VPHBSINN", ppu.ctrl.bits());
                            text_bitflags(ui, "MASK", "BGRsbMmG", ppu.mask.bits());
                            text_bitflags(ui, "STATUS", "VSO-----", ppu.status.bits());
                            ui.text(format!("SCROLL: {:02X} {:02X}", ppu.scroll.scroll_x, ppu.scroll.scroll_y));
                            ui.text(format!("ADDR: {:04X}", ppu.addr.get()));
                        }

                        fn text_bitflags(ui: &imgui::Ui, name: &str, labels: &str, bits: u8) {
                            ui.text(format!("{}: {:02X}", name, bits));

                            for (i, ch) in labels.chars().enumerate() {
                                ui.same_line();

                                if (bits & (1 << (7 - i))) != 0 {
                                    ui.text_colored([1.0, 0.0, 0.0, 1.0], ch.to_string());
                                } else {
                                    ui.text(ch.to_string());
                                }
                            }
                        }
                    });
            }

            unsafe {
                imgui_renderer.gl_context().clear_color(0.0, 0.0, 0.0, 1.0);
                imgui_renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
            }

            let seed = cpu.bus.borrow_mut().ppu.frame;

            let mut nes_frame_data = vec![0; NES_WIDTH as usize * NES_HEIGHT as usize * 3];
            for (i, frame_data) in nes_frame_data.iter_mut().enumerate() {
                *frame_data = ((i as u64 * 2654435761) ^ (seed * 1103515245)) as u8;
            }

            let gl = imgui_renderer.gl_context();
            unsafe {
                gl.bind_texture(glow::TEXTURE_2D, Some(nes_texture));
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    NES_WIDTH as i32,
                    NES_HEIGHT as i32,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(&nes_frame_data),
                );
            }

            let draw_data = imgui.render();
            imgui_renderer.render(draw_data).unwrap();

            window.gl_swap_window();
        },
    );
}

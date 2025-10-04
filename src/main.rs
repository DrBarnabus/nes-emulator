pub mod bus;
pub mod cpu;
pub mod emulator;
pub mod joypad;
pub mod ppu;
pub mod rom;

use clap::Parser;
use emulator::Emulator;
use glow::HasContext;
use imgui::{Condition, Context};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use joypad::JoypadButton;
use ppu::render::frame::Frame;
use rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

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
    let nes_texture = create_rgb_texture(gl, NES_WIDTH as i32, NES_HEIGHT as i32);
    let palette_texture = create_rgb_texture(gl, 8, 4);
    let pattern_table_textures = [create_rgb_texture(gl, 128, 128), create_rgb_texture(gl, 128, 128)];

    let mut debug_visible = args.debug;
    let mut active_palette: u8 = 0;

    let mut frame = Frame::new();

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
                    Event::KeyDown { keycode: Some(keycode), .. } => match keycode {
                        Keycode::F12 => debug_visible = !debug_visible,
                        Keycode::P => active_palette = (active_palette + 1) & 0x07,
                        _ => {
                            if let Some(button) = KEY_MAP.get(&keycode) {
                                cpu.bus.borrow_mut().joypad_1.set_button_state(*button, true)
                            }
                        }
                    },
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        if let Some(button) = KEY_MAP.get(&keycode) {
                            cpu.bus.borrow_mut().joypad_1.set_button_state(*button, false)
                        }
                    }
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

                        if ui.collapsing_header("Input", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                            text_bitflags_long(
                                ui,
                                "Joypad 1",
                                ["R", "L", "D", "U", "START", "SELECT", "B", "A"],
                                cpu.bus.borrow().joypad_1.button_states.bits(),
                            );
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

                            ui.separator();

                            ui.text("Palettes:");
                            draw_palettes(ui, palette_texture, active_palette);

                            ui.text("Pattern Tables:");
                            draw_pattern_table(ui, pattern_table_textures[0], 128.0 * 2.5);
                            draw_pattern_table(ui, pattern_table_textures[1], 128.0 * 2.5);
                        }
                    });
            }

            let gl = imgui_renderer.gl_context();
            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }

            let ppu = &cpu.bus.borrow_mut().ppu;
            ppu::render::render(ppu, &mut frame);
            update_rgb_texture(gl, nes_texture, NES_WIDTH as i32, NES_HEIGHT as i32, &frame.data);

            if debug_visible {
                let palette_data = populate_palette_texture(&ppu.palette_table);
                update_rgb_texture(gl, palette_texture, 8, 4, &palette_data);

                for (i, pattern_table_texture) in pattern_table_textures.iter().enumerate() {
                    let pattern_table_data = populate_pattern_table_texture(i, &ppu.chr_rom, &ppu.palette_table, active_palette);
                    update_rgb_texture(gl, *pattern_table_texture, 128, 128, &pattern_table_data);
                }
            }

            let draw_data = imgui.render();
            imgui_renderer.render(draw_data).unwrap();

            window.gl_swap_window();
        },
    );
}

fn create_rgb_texture(gl: &glow::Context, width: i32, height: i32) -> glow::Texture {
    unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::SRGB8 as i32, width, height, 0, glow::RGB, glow::UNSIGNED_BYTE, None);

        texture
    }
}

fn update_rgb_texture(gl: &glow::Context, texture: glow::Texture, width: i32, height: i32, data: &[u8]) {
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            width,
            height,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(data),
        );
    }
}

fn populate_palette_texture(palette_table: &[u8; 32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 * 4 * 3);

    for color_idx in 0..4 {
        for palette_idx in 0..8 {
            let palette_entry = palette_table[(palette_idx << 2) + color_idx];
            let color = ppu::render::palette::SYSTEM_PALETTE_COLOURS[palette_entry as usize];
            data.push(color.0);
            data.push(color.1);
            data.push(color.2);
        }
    }

    data
}

fn populate_pattern_table_texture(pattern_table_idx: usize, chr_rom: &[u8], palette_table: &[u8; 32], active_palette: u8) -> Vec<u8> {
    let mut data = vec![0; 128 * 128 * 3];

    let bank = pattern_table_idx * 0x1000;

    for tile_n in 0..256 {
        let tile_y = (tile_n / 16) * 8;
        let tile_x = (tile_n % 16) * 8;

        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut lower = tile[y];
            let mut upper = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper >>= 1;
                lower >>= 1;

                let palette_idx = (active_palette * 4 + value) as usize;
                let palette_entry = palette_table[palette_idx];
                let colour = ppu::render::palette::SYSTEM_PALETTE_COLOURS[palette_entry as usize];

                let base = (tile_y + y) * 3 * 128 + (tile_x + x) * 3;
                data[base] = colour.0;
                data[base + 1] = colour.1;
                data[base + 2] = colour.2;
            }
        }
    }

    data
}

fn draw_palettes(ui: &imgui::Ui, palette_texture: glow::Texture, active_palette: u8) {
    const PALETTE_WIDTH: f32 = 16.0;
    const PALETTE_HEIGHT: f32 = PALETTE_WIDTH * 4.0;

    let texture_id = imgui::TextureId::new(palette_texture.0.get() as usize);

    let [cursor_start_x, cursor_start_y] = ui.cursor_screen_pos();
    for p in 0..8 {
        let x_offset = p as f32 * (PALETTE_WIDTH + 2.0);
        ui.set_cursor_screen_pos([cursor_start_x + x_offset, cursor_start_y]);

        imgui::Image::new(texture_id, [PALETTE_WIDTH, PALETTE_HEIGHT])
            .uv0([p as f32 / 8.0, 0.0])
            .uv1([(p + 1) as f32 / 8.0, 1.0])
            .build(ui);

        if p == active_palette {
            let draw_list = ui.get_window_draw_list();
            let pos = [cursor_start_x + x_offset, cursor_start_y];
            draw_list
                .add_rect(pos, [pos[0] + PALETTE_WIDTH, pos[1] + PALETTE_HEIGHT], [1.0, 1.0, 0.0, 1.0])
                .thickness(2.0)
                .build();
        }
    }

    ui.set_cursor_screen_pos([cursor_start_x, cursor_start_y + PALETTE_HEIGHT + 4.0]);
}

fn draw_pattern_table(ui: &imgui::Ui, texture: glow::Texture, size: f32) {
    let texture_id = imgui::TextureId::new(texture.0.get() as usize);
    imgui::Image::new(texture_id, [size, size]).build(ui);
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

fn text_bitflags_long(ui: &imgui::Ui, name: &str, labels: [&str; 8], bits: u8) {
    ui.text(format!("{}: {:02X}", name, bits));

    for (i, str) in labels.iter().enumerate() {
        ui.same_line();

        if (bits & (1 << (7 - i))) != 0 {
            ui.text_colored([1.0, 0.0, 0.0, 1.0], str);
        } else {
            ui.text(str);
        }
    }
}

lazy_static::lazy_static! {
    pub static ref KEY_MAP: HashMap<Keycode, JoypadButton> = {
        let mut map = HashMap::new();
        map.insert(Keycode::S, JoypadButton::A);
        map.insert(Keycode::A, JoypadButton::B);
        map.insert(Keycode::Q, JoypadButton::SELECT);
        map.insert(Keycode::W, JoypadButton::START);
        map.insert(Keycode::Up, JoypadButton::UP);
        map.insert(Keycode::Down, JoypadButton::DOWN);
        map.insert(Keycode::Left, JoypadButton::LEFT);
        map.insert(Keycode::Right, JoypadButton::RIGHT);

        map
    };
}

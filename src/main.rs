#![windows_subsystem = "windows"]

pub mod bus;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod emulator;
pub mod ppu;

use cartridge::Cartridge;
use clap::Parser;
use controller::ControllerButton;
use emulator::Emulator;
use glow::HasContext;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use imgui::{Condition, Context, FontSource};
use imgui_glow_renderer::AutoRenderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use ppu::render::frame::Frame;
use raw_window_handle::HasWindowHandle;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::Window;
use anyhow::{Context as _, Result};

const TITLE: &str = "NES Emulator";
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

fn create_window(debug: bool) -> Result<(EventLoop<()>, Window, Surface<WindowSurface>, PossiblyCurrentContext)> {
    let event_loop = EventLoop::new().context("Failed to create event loop")?;

    let (window_width, window_height) = (
        if debug {
            DEFAULT_WINDOW_WIDTH + (DEFAULT_WINDOW_WIDTH / 2) // Allow enough room for the original width + 33% for a debug window
        } else {
            DEFAULT_WINDOW_WIDTH
        },
        DEFAULT_WINDOW_HEIGHT,
    );

    let window_attributes = Window::default_attributes()
        .with_title(TITLE)
        .with_inner_size(PhysicalSize::new(window_width, window_height))
        .with_resizable(true);

    let (window, config) = DisplayBuilder::new()
        .with_window_attributes(Some(window_attributes))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| configs.next().unwrap())
        .unwrap();

    let window = window.unwrap();

    let context_attributes = ContextAttributesBuilder::new().build(Some(window.window_handle()?.as_raw()));
    let context = unsafe { config.display().create_context(&config, &context_attributes).context("Failed to create context")? };

    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(Some(true)).build(
        window.window_handle()?.as_raw(),
        NonZeroU32::new(window_width).unwrap(),
        NonZeroU32::new(window_height).unwrap(),
    );

    let surface = unsafe { config.display().create_window_surface(&config, &surface_attributes).context("Failed to create surface")? };

    let context = context.make_current(&surface).context("Failed to make context current")?;

    Ok((event_loop, window, surface, context))
}

fn imgui_init(window: &Window) -> (WinitPlatform, Context) {
    let mut context = Context::create();
    context.set_ini_filename(None);
    context.style_mut().use_dark_colors();

    let mut platform = WinitPlatform::new(&mut context);
    platform.attach_window(context.io_mut(), window, HiDpiMode::Rounded);

    context.fonts().add_font(&[FontSource::DefaultFontData { config: None }]);
    context.io_mut().font_global_scale = (1.0 / platform.hidpi_factor()) as f32;

    (platform, context)
}

fn main() -> Result<()> {
    let args = Args::try_parse().context("Failed to parse command line arguments")?;

    let (mut event_loop, window, surface, gl_context) = create_window(args.debug).context("Failed to create window")?;
    let (mut imgui_platform, mut imgui_context) = imgui_init(&window);

    let gl = unsafe { glow::Context::from_loader_function_cstr(|s| gl_context.display().get_proc_address(s).cast()) };
    let mut imgui_renderer = AutoRenderer::new(gl, &mut imgui_context).context("Failed to create ImGui renderer")?;

    let gl = imgui_renderer.gl_context();
    let nes_texture = create_rgb_texture(gl, NES_WIDTH as i32, NES_HEIGHT as i32);
    let palette_texture = create_rgb_texture(gl, 8, 4);
    let pattern_table_textures = [create_rgb_texture(gl, 128, 128), create_rgb_texture(gl, 128, 128)];

    let mut debug_visible = args.debug;
    let mut active_palette: u8 = 0;

    let mut frame = Frame::new();

    let cartridge = Cartridge::load(args.rom.as_str()).context("Failed to load ROM file into Cartridge")?;

    let mut emulator = Emulator::new(cartridge);
    emulator.run(
        |_cpu| {
            // Per-instruction callback, which can be used for debugging/tracing
        },
        |cpu| {
            let mut should_exit = false;

            #[allow(deprecated)]
            event_loop.pump_events(Some(Duration::ZERO), |event, _window_target| {
                imgui_platform.handle_event(imgui_context.io_mut(), &window, &event);

                if let Event::WindowEvent { event: window_event, .. } = event {
                    match window_event {
                        WindowEvent::CloseRequested => should_exit = true,
                        WindowEvent::KeyboardInput { event: key_event, .. } => {
                            if let PhysicalKey::Code(keycode) = key_event.physical_key {
                                let is_pressed = key_event.state == ElementState::Pressed;

                                if is_pressed {
                                    match keycode {
                                        KeyCode::Escape => should_exit = true,
                                        KeyCode::F12 => debug_visible = !debug_visible,
                                        KeyCode::KeyP => active_palette = (active_palette + 1) & 0x07,
                                        _ => {}
                                    }
                                }

                                if let Some(button) = KEY_MAP.get(&keycode) {
                                    cpu.bus.borrow_mut().controller_1.set_button_state(*button, is_pressed)
                                }
                            }
                        }
                        WindowEvent::Resized(physical_size) => surface.resize(
                            &gl_context,
                            NonZeroU32::new(physical_size.width).unwrap(),
                            NonZeroU32::new(physical_size.height).unwrap(),
                        ),
                        _ => {}
                    }
                }
            });

            if should_exit {
                std::process::exit(0);
            }

            imgui_platform.prepare_frame(imgui_context.io_mut(), &window).unwrap();
            let ui = imgui_context.frame();

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
                                "Controller 1",
                                ["R", "L", "D", "U", "START", "SELECT", "B", "A"],
                                cpu.bus.borrow().controller_1.button_states.bits(),
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
                            let cpu = cpu.bus.borrow_mut();
                            let ppu = cpu.ppu.borrow_mut();
                            ui.text(format!("Cycle: {}", ppu.cycle));
                            ui.text(format!("Scanline: {}", ppu.scanline));
                            ui.text(format!("Frame: {}", ppu.frame));

                            ui.separator();

                            text_bitflags(ui, "CTRL", "VPHBSINN", ppu.ctrl.bits());
                            text_bitflags(ui, "MASK", "BGRsbMmG", ppu.mask.bits());
                            text_bitflags(ui, "STATUS", "VSO-----", ppu.status.bits());
                            ui.text(format!("SCROLL: {:02X} {:02X}", ppu.render_scroll_x, ppu.render_scroll_y));
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

            let cpu = cpu.bus.borrow_mut();
            let ppu = cpu.ppu.borrow_mut();
            ppu::render::render(&ppu, &mut frame);
            update_rgb_texture(gl, nes_texture, NES_WIDTH as i32, NES_HEIGHT as i32, &frame.data);

            if debug_visible {
                let palette_data = populate_palette_texture(&ppu.palette_table);
                update_rgb_texture(gl, palette_texture, 8, 4, &palette_data);

                for (i, pattern_table_texture) in pattern_table_textures.iter().enumerate() {
                    let pattern_table_data = populate_pattern_table_texture(i, &ppu.cartridge.borrow_mut().chr_rom, &ppu.palette_table, active_palette);
                    update_rgb_texture(gl, *pattern_table_texture, 128, 128, &pattern_table_data);
                }
            }

            let draw_data = imgui_context.render();
            imgui_renderer.render(draw_data).unwrap();

            surface.swap_buffers(&gl_context).unwrap();
        },
    );

    Ok(())
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

fn get_palette_color(palette_table: &[u8; 32], index: usize) -> u8 {
    let mirrored_index = match index {
        0x10 => 0x00,
        0x14 => 0x04,
        0x18 => 0x08,
        0x1C => 0x0C,
        _ if index < 32 => index,
        _ => 0,
    };

    palette_table[mirrored_index]
}

fn populate_palette_texture(palette_table: &[u8; 32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 * 4 * 3);

    for color_idx in 0..4 {
        for palette_idx in 0..8 {
            let index = (palette_idx << 2) + color_idx;
            let palette_entry = get_palette_color(palette_table, index);
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
                let palette_entry = get_palette_color(palette_table, palette_idx);
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
    pub static ref KEY_MAP: HashMap<KeyCode, ControllerButton> = {
        let mut map = HashMap::new();
        map.insert(KeyCode::KeyS, ControllerButton::A);
        map.insert(KeyCode::KeyA, ControllerButton::B);
        map.insert(KeyCode::KeyQ, ControllerButton::SELECT);
        map.insert(KeyCode::KeyW, ControllerButton::START);
        map.insert(KeyCode::ArrowUp, ControllerButton::UP);
        map.insert(KeyCode::ArrowDown, ControllerButton::DOWN);
        map.insert(KeyCode::ArrowLeft, ControllerButton::LEFT);
        map.insert(KeyCode::ArrowRight, ControllerButton::RIGHT);

        map
    };
}

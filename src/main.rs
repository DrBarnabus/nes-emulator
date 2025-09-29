pub mod bus;
pub mod cpu;
pub mod emulator;
pub mod rom;

use cpu::Cpu;
use cpu::mem::Mem;
use cpu::trace::trace;
use emulator::Emulator;
use rand::Rng;
use rom::Rom;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

fn color(byte: u8) -> Color {
    match byte {
        0 => Color::BLACK,
        1 => Color::WHITE,
        2 | 9 => Color::GREY,
        3 | 10 => Color::RED,
        4 | 11 => Color::GREEN,
        5 | 12 => Color::BLUE,
        6 | 13 => Color::MAGENTA,
        7 | 14 => Color::YELLOW,
        _ => Color::CYAN,
    }
}

fn read_screen_state(cpu: &Cpu, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_index = 0;
    let mut update = false;

    for i in 0x0200..0x600 {
        let color_index = cpu.read(i as u16);
        let (r, g, b) = color(color_index).rgb();

        if frame[frame_index] != r || frame[frame_index + 1] != g || frame[frame_index + 2] != b {
            frame[frame_index] = r;
            frame[frame_index + 1] = g;
            frame[frame_index + 2] = b;
            update = true;
        }

        frame_index += 3;
    }

    update
}

fn handle_user_input(cpu: &mut Cpu, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                cpu.write(0xff, 0x77);
            }
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                cpu.write(0xff, 0x73);
            }
            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                cpu.write(0xff, 0x61);
            }
            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                cpu.write(0xff, 0x64);
            }
            _ => {}
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("6502 Snake", 32 * 10, 32 * 10).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let mut screen_state = [0u8; 32 * 3 * 32];
    let mut rng = rand::rng();

    let rom = Rom::load("snake.nes").unwrap();
    let mut emulator = Emulator::new(rom);
    emulator.run(|cpu| {
        println!("{}", trace(cpu));

        cpu.write(0xfe, rng.random_range(1..16));

        handle_user_input(cpu, &mut event_pump);

        if read_screen_state(cpu, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
        }
        canvas.present();
    });
}

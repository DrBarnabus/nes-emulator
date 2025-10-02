pub mod bus;
pub mod cpu;
pub mod emulator;
pub mod ppu;
pub mod rom;

use emulator::Emulator;
use rom::Rom;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn handle_user_input(event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            _ => {}
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    const SCALE: u32 = 3;
    let window = video_subsystem
        .window("NES Emulator", 256 * SCALE, 240 * SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let rom = Rom::load("test_roms/Donkey Kong (World) (Rev A).nes").unwrap();
    let mut emulator = Emulator::new(rom);

    let mut frame_count = 0u32;
    let mut last_time = std::time::Instant::now();

    emulator.run(
        |_cpu| {
            // Per-instruction callback, which can be used for debugging/tracing
        },
        |cpu| {
            handle_user_input(&mut event_pump);
            canvas.present();

            frame_count += 1;
            let elapsed = last_time.elapsed();

            // Update FPS counter every second
            if elapsed.as_secs() >= 1 {
                let fps = frame_count as f64 / elapsed.as_secs_f64();
                canvas
                    .window_mut()
                    .set_title(&format!("NES Emulator - FPS: {:.2} - CPU Cycle: {}", fps, cpu.cycles))
                    .unwrap();

                frame_count = 0;
                last_time = std::time::Instant::now();
            }
        },
    );
}

use super::bus::Bus;
use super::cpu::Cpu;
use super::rom::Rom;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct TimingController {
    start_time: Instant,
    emulated_cycles: u64,
    nanoseconds_per_cycle: f64,
}

impl TimingController {
    pub fn new() -> Self {
        const NTSC_CPU_FREQUENCY: f64 = 1_789_773.0;
        const FREQUENCY_MULTIPLIER: f64 = 0.032; // TODO: This is temporary to get `snake.nes` to run at a reasonable speed.

        Self {
            start_time: Instant::now(),
            emulated_cycles: 0,
            nanoseconds_per_cycle: 1_000_000_000.0 / (NTSC_CPU_FREQUENCY * FREQUENCY_MULTIPLIER),
        }
    }

    pub fn synchronize(&mut self, cycles: u64) {
        self.emulated_cycles += cycles;

        let target_ns = (self.emulated_cycles as f64 * self.nanoseconds_per_cycle) as u64;
        let target = Duration::from_nanos(target_ns);
        let elapsed = self.start_time.elapsed();

        if elapsed < target {
            let sleep_time = target - elapsed;

            if sleep_time > Duration::from_millis(1) {
                std::thread::sleep(sleep_time);
            } else {
                while self.start_time.elapsed() < target {
                    std::hint::spin_loop();
                }
            }
        }
    }
}

impl Default for TimingController {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Emulator {
    #[allow(dead_code)]
    bus: Rc<RefCell<Bus>>,
    cpu: Cpu,
}

impl Emulator {
    pub fn new(rom: Rom) -> Self {
        let bus = Rc::new(RefCell::new(Bus::new(rom)));
        let cpu = Cpu::new(Rc::clone(&bus));

        Self { bus, cpu }
    }

    pub fn run<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        self.cpu.reset();

        const SYNC_THRESHOLD: u64 = 1000;
        let mut timing_controller = TimingController::default();
        let mut accumulated_cycles = 0u64;

        loop {
            if self.cpu.halted {
                break;
            }

            callback(&mut self.cpu);

            let cpu_cycles = self.cpu.step();
            accumulated_cycles += cpu_cycles;

            if accumulated_cycles >= SYNC_THRESHOLD {
                timing_controller.synchronize(accumulated_cycles);
                accumulated_cycles = 0;
            }
        }
    }
}

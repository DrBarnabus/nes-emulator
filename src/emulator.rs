use super::bus::Bus;
use super::cartridge::Cartridge;
use super::cpu::Cpu;
use crate::apu::Apu;
use crate::audio::AudioOutput;
use crate::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub const NTSC_CPU_FREQUENCY: f64 = 1_789_773.0;

pub struct TimingController {
    start_time: Instant,
    emulated_cycles: u64,
    nanoseconds_per_cycle: f64,
}

impl TimingController {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            emulated_cycles: 0,
            nanoseconds_per_cycle: 1_000_000_000.0 / NTSC_CPU_FREQUENCY,
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
    pub cpu: Cpu,
    pub ppu: Rc<RefCell<Ppu>>,
    pub apu: Rc<RefCell<Apu>>,
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub bus: Rc<RefCell<Bus>>,

    pub audio: Option<AudioOutput>,
}

impl Emulator {
    pub fn new(cartridge: Cartridge) -> Self {
        let cartridge = Rc::new(RefCell::new(cartridge));
        let ppu = Rc::new(RefCell::new(Ppu::new(Rc::clone(&cartridge))));
        let apu = Rc::new(RefCell::new(Apu::new()));
        let bus = Rc::new(RefCell::new(Bus::new(Rc::clone(&ppu), Rc::clone(&apu), Rc::clone(&cartridge))));
        let cpu = Cpu::new(Rc::clone(&bus));

        Emulator {
            cpu,
            ppu,
            apu,
            cartridge,
            bus,
            audio: None,
        }
    }

    pub fn connect_audio(&mut self, audio: AudioOutput) {
        self.audio = Some(audio);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();

        // Tick PPU for the CPU reset cycles (7 CPU cycles = 21 PPU cycles)
        for _ in 0..21 {
            self.ppu.borrow_mut().tick();
        }

        // Tick APU for the CPU reset cycles (7 CPU cycles)
        for _ in 0..7 {
            self.apu.borrow_mut().clock();
        }
    }

    pub fn run_frame(&mut self) -> u64 {
        let mut accumulated_cycles = 0;

        loop {
            let cpu_cycles = self.cpu.step();
            accumulated_cycles += cpu_cycles;

            let mut frame_complete = false;
            for _ in 0..cpu_cycles * 3 {
                let mut ppu = self.ppu.borrow_mut();
                if ppu.tick() {
                    frame_complete = true;
                }

                if ppu.poll_nmi() {
                    self.bus.borrow_mut().trigger_nmi();
                }
            }

            for _ in 0..cpu_cycles {
                let mut apu = self.apu.borrow_mut();
                apu.clock();

                let apu_sample = apu.filtered_output();
                if let Some(audio) = &mut self.audio {
                    audio.push_source_sample(apu_sample);
                }

                if apu.irq_pending() {
                    self.bus.borrow_mut().trigger_irq();
                }
            }

            if frame_complete {
                break;
            }
        }

        accumulated_cycles
    }

    pub fn run<F>(&mut self, mut frame_callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        self.reset();

        const SYNC_THRESHOLD: u64 = 1000;
        let mut timing_controller = TimingController::default();
        let mut accumulated_cycles = 0u64;

        loop {
            if self.cpu.halted {
                break;
            }

            accumulated_cycles += self.run_frame();
            frame_callback(&mut self.cpu);

            if accumulated_cycles >= SYNC_THRESHOLD {
                timing_controller.synchronize(accumulated_cycles);
                accumulated_cycles = 0;
            }
        }
    }
}

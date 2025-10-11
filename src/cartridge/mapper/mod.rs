mod mapper_000;

pub use mapper_000::Mapper000;

use super::Mirroring;

pub enum MappedRead {
    Data(u8),
    PrgRom(u16),
    PrgRam(u16),
    None,
}

pub enum MappedWrite {
    PrgRam(u16),
    None,
}

pub trait Mapper {
    fn cpu_read(&mut self, address: u16) -> MappedRead;

    fn cpu_write(&mut self, address: u16, value: u8) -> MappedWrite;

    fn ppu_read(&mut self, address: u16) -> usize;

    fn ppu_write(&mut self, address: u16, value: u8) -> Option<usize>;

    fn mirroring(&self) -> Mirroring;
}

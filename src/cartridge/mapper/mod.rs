mod mapper_000;

pub use mapper_000::Mapper000;

use super::Mirroring;

pub enum MappedRead {
    Data(u8),
    Address(u16),
    None,
}

pub enum MappedWrite {
    Address(u16),
    PrgRam(u16),
    None,
}

pub trait Mapper {
    fn cpu_read(&mut self, address: u16) -> MappedRead;

    fn cpu_write(&mut self, address: u16, value: u8) -> MappedWrite;

    fn ppu_read(&mut self, address: u16) -> MappedRead;

    fn ppu_write(&mut self, address: u16, value: u8) -> MappedWrite;

    fn mirroring(&self) -> Mirroring;
}

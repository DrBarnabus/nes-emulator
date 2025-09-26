const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // NES^Z
const PRG_ROM_PAGE_SIZE: usize = 16384; // 16KB
const CHR_ROM_PAGE_SIZE: usize = 8192; // 8KB

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw: &[u8]) -> Result<Rom, String> {
        if raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let version = (raw[7] >> 2) & 0b11;
        if version != 0 {
            return Err(format!("Unsupported iNES version: {}", version));
        }

        let skip_trainer = raw[6] & 0x4 != 0;

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let mapper = (raw[7] & 0xF0) | (raw[6] >> 4);

        let nametable_arrangement = raw[6] & 0x1 != 0;
        let alternative_nametable_layout = raw[6] & 0x8 != 0;
        let screen_mirroring = match (nametable_arrangement, alternative_nametable_layout) {
            (_, true) => Mirroring::FourScreen,
            (true, false) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            screen_mirroring,
        })
    }

    pub fn load(path: &str) -> Result<Rom, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        Rom::new(&bytes)
    }
}

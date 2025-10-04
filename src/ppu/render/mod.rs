pub mod frame;
pub mod palette;

use crate::ppu::Ppu;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::palette::SYSTEM_PALETTE_COLOURS;

fn background_palette(ppu: &Ppu, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x3C0 + attr_table_idx];

    let palette_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => (attr_byte & 0x03) as usize,
        (1, 0) => ((attr_byte >> 2) & 0x03) as usize,
        (0, 1) => ((attr_byte >> 4) & 0x03) as usize,
        (1, 1) => ((attr_byte >> 6) & 0x03) as usize,
        _ => unreachable!("Invalid tile coordinate: ({}, {})", tile_column, tile_row),
    };

    let palette_start = 1 + palette_idx * 4;
    [
        ppu.palette_table[0],
        ppu.palette_table[palette_start],
        ppu.palette_table[palette_start + 1],
        ppu.palette_table[palette_start + 2],
    ]
}

pub fn render(ppu: &Ppu, frame: &mut Frame) {
    let bank = ppu.ctrl.background_pattern_addr();

    for i in 0..0x3C0 {
        let tile = ppu.vram[i] as u16;
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 16) as usize];
        let palette = background_palette(ppu, tile_column, tile_row);

        for y in 0..=7 {
            let mut lower = tile[y];
            let mut upper = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (upper & 1) << 1 | (lower & 1);
                lower >>= 1;
                upper >>= 1;

                let colour = match value {
                    0 => SYSTEM_PALETTE_COLOURS[palette[0] as usize],
                    1 => SYSTEM_PALETTE_COLOURS[palette[1] as usize],
                    2 => SYSTEM_PALETTE_COLOURS[palette[2] as usize],
                    3 => SYSTEM_PALETTE_COLOURS[palette[3] as usize],
                    _ => unreachable!("Invalid value: {}", value),
                };

                frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, colour);
            }
        }
    }
}

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

fn sprite_palette(ppu: &Ppu, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + palette_idx as usize * 4;
    [0, ppu.palette_table[start], ppu.palette_table[start + 1], ppu.palette_table[start + 2]]
}

pub fn render(ppu: &Ppu, frame: &mut Frame) {
    let background_bank = ppu.ctrl.background_pattern_addr();
    let sprite_bank = ppu.ctrl.sprite_pattern_addr();

    // Background
    for i in 0..0x3C0 {
        let tile = ppu.vram[i] as u16;
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile = &ppu.chr_rom[(background_bank + tile * 16) as usize..=(background_bank + tile * 16 + 15) as usize];
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

    // Sprites
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let tile_x = ppu.oam_data[i + 3] as usize;
        let tile_y = ppu.oam_data[i] as usize;
        let tile = &ppu.chr_rom[(sprite_bank + tile_idx * 16) as usize..=(sprite_bank + tile_idx * 16 + 15) as usize];

        let flip_vertical = ppu.oam_data[i + 2] >> 7 & 1 == 1;
        let flip_horizontal = ppu.oam_data[i + 2] >> 6 & 1 == 1;

        let palette_idx = ppu.oam_data[i + 2] & 0x03;
        let sprite_palette = sprite_palette(ppu, palette_idx);

        for y in 0..=7 {
            let mut lower = tile[y];
            let mut upper = tile[y + 8];

            'inner_loop: for x in (0..=7).rev() {
                let value = (upper & 1) << 1 | (lower & 1);
                lower >>= 1;
                upper >>= 1;

                let colour = match value {
                    0 => continue 'inner_loop,
                    1 => SYSTEM_PALETTE_COLOURS[sprite_palette[1] as usize],
                    2 => SYSTEM_PALETTE_COLOURS[sprite_palette[2] as usize],
                    3 => SYSTEM_PALETTE_COLOURS[sprite_palette[3] as usize],
                    _ => unreachable!("Invalid value: {}", value),
                };

                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, colour),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, colour),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, colour),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, colour),
                }
            }
        }
    }
}

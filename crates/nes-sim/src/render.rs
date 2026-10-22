//! Picture renderer for the test machine: turns VRAM, OAM, palette RAM and the logged
//! PPUCTRL / PPUSCROLL writes into a 256x240 frame of NES colour indices (0-63), so the
//! result can be compared pixel by pixel with another emulator.
//!
//! Model: 8x8 sprites, CHR read through the mapper's banking, header mirroring (vertical or horizontal; vertical scroll ignores the nametable bit), no emphasis or greyscale. The horizontal
//! scroll for line y is whatever was written before dot 257 of the line above (when the
//! PPU copies it); the vertical scroll is taken at the pre-render line.

use crate::nes::NesBus;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

#[derive(Clone, Copy, Default)]
struct Regs {
    ctrl: u8,
    sx: u8,
    sy: u8,
}

/// Register state after replaying every logged write made before `cycle`.
fn regs_at(log: &[(u64, u8, u8)], cycle: u64) -> Regs {
    let mut r = Regs::default();
    for &(c, kind, v) in log {
        if c >= cycle {
            break;
        }
        match kind {
            0 => r.ctrl = v,
            1 => r.sx = v,
            _ => r.sy = v,
        }
    }
    r
}

fn palette(bus: &NesBus, addr: u16) -> u8 {
    bus.vram[NesBus::vram_index(addr)] & 0x3F
}

/// Background colour number (0-3) and palette (0-3) at screen (x, y).
fn bg_pixel(bus: &NesBus, ctrl: u8, sx: u8, sy: u8, x: usize, y: usize) -> (u8, u8) {
    let wx = (ctrl as usize & 1) * 256 + sx as usize + x;
    let mut wy = sy as usize + y;
    if wy >= 240 {
        wy -= 240;
    }
    let nt = bus.nt_base(wx);
    let (tx, ty) = ((wx & 0xFF) / 8, wy / 8);
    let tile = bus.vram[nt + ty * 32 + tx] as usize;
    let attr = bus.vram[nt + 0x3C0 + (ty / 4) * 8 + tx / 4];
    let shift = ((ty % 4) / 2) * 4 + ((tx % 4) / 2) * 2;
    let pal = (attr >> shift) & 3;
    let base = if ctrl & 0x10 != 0 { 0x1000 } else { 0 };
    let a = base + tile * 16 + wy % 8;
    let bit = 7 - (wx % 8);
    let c = ((bus.chr_peek(a as u16) >> bit) & 1) | ((bus.chr_peek(a as u16 + 8) >> bit) & 1) << 1;
    (c, pal)
}

/// First sprite (lowest OAM index) with an opaque pixel at (x, y):
/// (colour 1-3, palette, behind background).
fn sprite_pixel(bus: &NesBus, ctrl: u8, x: usize, y: usize) -> Option<(u8, u8, bool)> {
    let base = if ctrl & 0x08 != 0 { 0x1000 } else { 0 };
    for s in bus.oam.chunks(4) {
        let (top, tile, attr, left) = (s[0] as usize + 1, s[1] as usize, s[2], s[3] as usize);
        if y < top || y >= top + 8 || x < left || x >= left + 8 {
            continue;
        }
        let row = if attr & 0x80 != 0 { 7 - (y - top) } else { y - top };
        let col = if attr & 0x40 != 0 { 7 - (x - left) } else { x - left };
        let a = base + tile * 16 + row;
        let bit = 7 - col;
        let c = ((bus.chr_peek(a as u16) >> bit) & 1) | ((bus.chr_peek(a as u16 + 8) >> bit) & 1) << 1;
        if c != 0 {
            return Some((c, attr & 3, attr & 0x20 != 0));
        }
    }
    None
}

/// Fine horizontal scroll (0-7) used on each line of frame `k`.
pub fn line_fine_x(bus: &NesBus, k: u64) -> Vec<u8> {
    use crate::nes::{CYCLES_PER_FRAME, VBLANK_CYCLES};
    let start = k * CYCLES_PER_FRAME + VBLANK_CYCLES;
    (0..HEIGHT).map(|y| regs_at(&bus.reg_log, start + (341 * y as u64 + 257) / 3).sx & 7).collect()
}

/// Renders frame `k` (0-based, counted from power-on) using the current VRAM/OAM.
pub fn frame(bus: &NesBus, k: u64) -> Vec<u8> {
    use crate::nes::{CYCLES_PER_FRAME, VBLANK_CYCLES};
    let start = k * CYCLES_PER_FRAME + VBLANK_CYCLES;
    // vertical scroll is copied during the pre-render line (dots 280-304)
    let vertical = regs_at(&bus.reg_log, start + 280 / 3);
    let mask = bus.ppumask;
    let universal = palette(bus, 0x3F00);
    let mut out = vec![universal; WIDTH * HEIGHT];
    for y in 0..HEIGHT {
        // horizontal scroll for line y is copied at dot 257 of the line before it
        let h = regs_at(&bus.reg_log, start + (341 * y as u64 + 257) / 3);
        for x in 0..WIDTH {
            let left = x < 8;
            let bg = if mask & 0x08 != 0 && !(left && mask & 0x02 == 0) {
                bg_pixel(bus, h.ctrl, h.sx, vertical.sy, x, y)
            } else {
                (0, 0)
            };
            let spr = if mask & 0x10 != 0 && !(left && mask & 0x04 == 0) { sprite_pixel(bus, h.ctrl, x, y) } else { None };
            out[y * WIDTH + x] = match (bg, spr) {
                ((0, _), Some((c, p, _))) | ((_, _), Some((c, p, false))) => palette(bus, 0x3F10 + p as u16 * 4 + c as u16),
                ((0, _), None) => universal,
                ((c, p), _) => palette(bus, 0x3F00 + p as u16 * 4 + c as u16),
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nes::{pixel_cycle, CYCLES_PER_FRAME};

    fn px(f: &[u8], x: usize, y: usize) -> u8 {
        f[y * WIDTH + x]
    }

    /// Palette RAM: universal $0F; background palettes 0-3 and sprite palettes 0-3
    /// get distinct colours so every lookup is checkable.
    fn bus() -> NesBus {
        let mut b = NesBus::new(vec![], vec![]);
        for (i, v) in (0..32).map(|i| 0x10 + i as u8).enumerate() {
            b.vram[0x3F00 + i] = v;
        }
        b.vram[0x3F00] = 0x0F;
        b.ppumask = 0x1E;
        b
    }

    #[test]
    fn background_tiles_attributes_and_universal_colour() {
        let mut b = bus();
        // tile 1: row 0 = colours 1,2,3,0,0,0,0,0
        b.vram[16] = 0b1010_0000;
        b.vram[16 + 8] = 0b0110_0000;
        b.vram[0x2000] = 1; // top-left cell
        b.vram[0x2000 + 2] = 1; // cell (2, 0): the byte's top-right quadrant
        b.vram[0x2000 + 4] = 1; // cell (4, 0): next attribute byte
        b.vram[0x23C0] = 0b10; // byte 0, top-left quadrant -> palette 2
        b.vram[0x23C1] = 0b01; // byte 1 -> palette 1
        let f = frame(&b, 0);
        assert_eq!([px(&f, 0, 0), px(&f, 1, 0), px(&f, 2, 0), px(&f, 3, 0)], [0x19, 0x1A, 0x1B, 0x0F]);
        assert_eq!(px(&f, 16, 0), 0x11, "top-right quadrant of byte 0 is palette 0");
        assert_eq!(px(&f, 32, 0), 0x15, "palette 1");
        assert_eq!(px(&f, 0, 1), 0x0F, "row 1 of tile 1 is blank");
        assert_eq!(px(&f, 200, 100), 0x0F, "tile 0 is blank");
    }

    #[test]
    fn sprites_flips_priority_and_order() {
        let mut b = bus();
        b.vram[32] = 0x80; // tile 2: top-left pixel, colour 1
        b.vram[48..56].fill(0xFF); // tile 3: solid colour 1
        b.vram[0x2000 + 10 * 32 + 10] = 3; // opaque background cell at (80, 80)
        b.oam[..4].copy_from_slice(&[19, 2, 0x00, 20]); // plain, sprite palette 0
        b.oam[4..8].copy_from_slice(&[39, 2, 0x41, 20]); // h-flip, palette 1
        b.oam[8..12].copy_from_slice(&[59, 2, 0x82, 20]); // v-flip, palette 2
        b.oam[12..16].copy_from_slice(&[79, 3, 0x20, 80]); // behind opaque background
        b.oam[16..20].copy_from_slice(&[99, 2, 0x00, 120]); // index 4 ...
        b.oam[20..24].copy_from_slice(&[99, 2, 0x03, 120]); // ... wins over index 5
        for i in 6..64 {
            b.oam[i * 4] = 0xFF;
        }
        let f = frame(&b, 0);
        assert_eq!(px(&f, 20, 20), 0x21, "sprite palette 0 colour 1");
        assert_eq!(px(&f, 27, 40), 0x25, "h-flip puts the pixel on the right");
        assert_eq!(px(&f, 20, 67), 0x29, "v-flip puts it on the bottom row");
        assert_eq!(px(&f, 80, 80), 0x11, "background wins over a behind-sprite");
        assert_eq!(px(&f, 120, 100), 0x21, "lower OAM index wins");
    }

    #[test]
    fn left_column_masking() {
        let mut b = bus();
        b.vram[16..24].fill(0xFF);
        b.vram[0x2000] = 1;
        b.ppumask = 0x18; // hide both in the leftmost 8 pixels
        let f = frame(&b, 0);
        assert_eq!(px(&f, 3, 3), 0x0F);
        assert_eq!(px(&f, 8, 3), 0x0F, "cell 1 is tile 0");
        b.ppumask = 0x1E;
        assert_eq!(px(&frame(&b, 0), 3, 3), 0x11);
    }

    #[test]
    fn horizontal_scroll_changes_mid_frame_from_the_next_line() {
        let mut b = bus();
        b.vram[16..24].fill(0xFF); // tile 1 solid colour 1 (palette 0 -> $11)
        b.vram[32 + 8..48].fill(0xFF); // tile 2 solid colour 2 (-> $12)
        b.vram[0x2000..0x23C0].fill(1);
        b.vram[0x2400..0x27C0].fill(2);
        let k = 3;
        let start = k * CYCLES_PER_FRAME;
        // NMI: nametable 0, scroll 0; split during line 30 -> nametable 1
        b.reg_log = vec![
            (start + 20, 0, 0x80),
            (start + 30, 1, 0),
            (start + 40, 2, 0),
            (start + pixel_cycle(250, 30) + 5, 0, 0x81),
        ];
        let f = frame(&b, k);
        assert_eq!(px(&f, 100, 31), 0x11, "line 31 copied before the write");
        assert_eq!(px(&f, 100, 32), 0x12, "line 32 uses the new nametable");
        // writes in an earlier frame still count as the starting state
        b.reg_log = vec![(10, 0, 0x81), (20, 1, 8), (30, 2, 0)];
        let f = frame(&b, k);
        assert_eq!(px(&f, 0, 0), 0x12);
    }

    #[test]
    fn vertical_scroll_is_latched_at_the_pre_render_line() {
        let mut b = bus();
        b.vram[16..24].fill(0xFF);
        b.vram[0x2000 + 2 * 32] = 1; // cell (0, 2) = rows 16-23
        let start = 2 * CYCLES_PER_FRAME;
        b.reg_log = vec![(start + 20, 0, 0x80), (start + 30, 1, 0), (start + 40, 2, 16)];
        let f = frame(&b, 2);
        assert_eq!(px(&f, 0, 0), 0x11, "scrolled up by 16");
        // a y write mid-frame does not move the picture
        b.reg_log.push((start + pixel_cycle(0, 100), 1, 0));
        b.reg_log.push((start + pixel_cycle(10, 100), 2, 0));
        assert_eq!(px(&frame(&b, 2), 0, 0), 0x11);
    }
}

//! Phase 7: CHR ROM Integration
//!
//! Extract CHR ROM and convert to NESL tile definitions

use std::fs;
use std::path::Path;

pub struct ChrExtractor {
    chr_rom: Vec<u8>,
}

impl ChrExtractor {
    pub fn from_rom(rom_path: &Path) -> Result<Self, String> {
        let rom_bytes = fs::read(rom_path)
            .map_err(|e| format!("Cannot read ROM: {}", e))?;

        if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
            return Err("Invalid NES ROM file".to_string());
        }

        let prg_banks = rom_bytes[4] as usize;
        let chr_banks = rom_bytes[5] as usize;

        if chr_banks == 0 {
            return Err("No CHR ROM (uses CHR RAM)".to_string());
        }

        let prg_size = prg_banks * 16384;
        let chr_offset = 16 + prg_size;
        let chr_size = chr_banks * 8192;

        if rom_bytes.len() < chr_offset + chr_size {
            return Err("ROM too small for CHR data".to_string());
        }

        let chr_rom = rom_bytes[chr_offset..chr_offset + chr_size].to_vec();

        Ok(Self { chr_rom })
    }

    /// Get number of tiles in CHR ROM
    pub fn tile_count(&self) -> usize {
        self.chr_rom.len() / 16
    }

    /// Extract a single tile (16 bytes -> 8x8 pixels)
    pub fn get_tile(&self, index: usize) -> Option<[[u8; 8]; 8]> {
        if index >= self.tile_count() {
            return None;
        }

        let offset = index * 16;
        let tile_data = &self.chr_rom[offset..offset + 16];

        let mut pixels = [[0u8; 8]; 8];

        for y in 0..8 {
            let plane0 = tile_data[y];
            let plane1 = tile_data[y + 8];

            for x in 0..8 {
                let bit_pos = 7 - x;
                let bit0 = (plane0 >> bit_pos) & 1;
                let bit1 = (plane1 >> bit_pos) & 1;
                pixels[y][x] = bit0 | (bit1 << 1);
            }
        }

        Some(pixels)
    }

    /// Generate NESL tile definition for a single tile
    pub fn tile_to_nesl(&self, index: usize, name: Option<&str>) -> String {
        let pixels = match self.get_tile(index) {
            Some(p) => p,
            None => return String::new(),
        };

        let default_name = format!("tile_{:03}", index);
        let tile_name = name.unwrap_or(&default_name);
        let mut code = format!("tile {} {{\n", tile_name);

        for row in &pixels {
            code.push_str("    ");
            for &pixel in row {
                code.push(match pixel {
                    0 => '.',
                    1 => '1',
                    2 => '2',
                    3 => '3',
                    _ => '?',
                });
            }
            code.push('\n');
        }

        code.push_str("}\n");
        code
    }

    /// Generate NESL code for all tiles
    pub fn generate_all_tiles(&self) -> String {
        let mut code = String::new();

        code.push_str("// ========== CHR ROM Tiles ==========\n");
        code.push_str("// Extracted from original ROM\n\n");

        let total = self.tile_count();
        eprintln!("Generating {} tiles...", total);

        // Generate tiles in batches
        let batch_size = 64;
        let batches = (total + batch_size - 1) / batch_size;

        for batch in 0..batches {
            let start = batch * batch_size;
            let end = (start + batch_size).min(total);

            code.push_str(&format!("\n// Tiles {}-{}\n\n", start, end - 1));

            for i in start..end {
                code.push_str(&self.tile_to_nesl(i, None));
                code.push('\n');
            }
        }

        code
    }

    /// Generate NESL code with smart naming for known games
    pub fn generate_smart_tiles(&self) -> String {
        let mut code = String::new();

        code.push_str("// ========== CHR ROM Tiles ==========\n\n");

        let total = self.tile_count();

        // For Mario: tiles 0-127 are background, 128-255 are sprites
        if total >= 256 {
            code.push_str("// Background tiles (0-127)\n\n");
            for i in 0..128 {
                let name = self.guess_tile_name(i, "bg");
                code.push_str(&self.tile_to_nesl(i, Some(&name)));
                code.push('\n');
            }

            code.push_str("\n// Sprite tiles (128-255)\n\n");
            for i in 128..256 {
                let name = self.guess_tile_name(i, "sprite");
                code.push_str(&self.tile_to_nesl(i, Some(&name)));
                code.push('\n');
            }
        } else {
            // Unknown game, use generic names
            for i in 0..total {
                code.push_str(&self.tile_to_nesl(i, None));
                code.push('\n');
            }
        }

        code
    }

    /// Guess a descriptive name for a tile
    fn guess_tile_name(&self, index: usize, category: &str) -> String {
        // Could be enhanced with pattern recognition
        // For now, just use category + number
        format!("{}_{:03}", category, index)
    }

    /// Check if a tile is blank (all zeros)
    pub fn is_blank(&self, index: usize) -> bool {
        if let Some(pixels) = self.get_tile(index) {
            pixels.iter().all(|row| row.iter().all(|&p| p == 0))
        } else {
            false
        }
    }

    /// Get tile statistics
    pub fn stats(&self) -> TileStats {
        let total = self.tile_count();
        let mut blank = 0;
        let mut used_pixels = [0; 4];

        for i in 0..total {
            if self.is_blank(i) {
                blank += 1;
            }

            if let Some(pixels) = self.get_tile(i) {
                for row in &pixels {
                    for &pixel in row {
                        if pixel < 4 {
                            used_pixels[pixel as usize] += 1;
                        }
                    }
                }
            }
        }

        TileStats {
            total,
            blank,
            non_blank: total - blank,
            pixel_usage: used_pixels,
        }
    }

    /// Export as binary CHR file
    pub fn save_chr(&self, path: &Path) -> Result<(), String> {
        fs::write(path, &self.chr_rom)
            .map_err(|e| format!("Cannot write CHR file: {}", e))
    }
}

#[derive(Debug)]
pub struct TileStats {
    pub total: usize,
    pub blank: usize,
    pub non_blank: usize,
    pub pixel_usage: [u64; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_extraction() {
        // Create a simple test tile
        let mut chr_rom = vec![0u8; 16];

        // Set some pixels
        chr_rom[0] = 0xFF; // Plane 0, row 0: all 1s
        chr_rom[8] = 0x00; // Plane 1, row 0: all 0s

        let extractor = ChrExtractor { chr_rom };
        let pixels = extractor.get_tile(0).unwrap();

        // First row should be all 1s (color 1)
        assert_eq!(pixels[0], [1, 1, 1, 1, 1, 1, 1, 1]);
    }
}

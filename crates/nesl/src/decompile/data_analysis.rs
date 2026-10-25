//! Phase 6: Data Segment Analysis
//!
//! Identify and extract data segments from ROM
//! - Level data
//! - Palettes
//! - Sprite definitions
//! - Music data

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Palette,           // 32 bytes palette data
    LevelData,         // Level layout
    SpriteMetadata,    // Sprite positions
    TileMap,           // Background tiles
    MusicData,         // APU music
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DataSegment {
    pub address: u16,
    pub size: usize,
    pub data_type: DataType,
    pub data: Vec<u8>,
    pub referenced_by: Vec<u16>,
}

pub struct DataAnalyzer {
    rom: Vec<u8>,
    segments: Vec<DataSegment>,
    code_references: HashMap<u16, Vec<u16>>,
}

impl DataAnalyzer {
    pub fn new(prg_rom: &[u8]) -> Self {
        Self {
            rom: prg_rom.to_vec(),
            segments: Vec::new(),
            code_references: HashMap::new(),
        }
    }

    /// Analyze ROM and identify data segments
    pub fn analyze(&mut self) {
        eprintln!("Phase 6: Analyzing data segments...");

        // 1. Find all data references in code
        self.find_data_references();

        // 2. Identify palettes (most reliable pattern)
        self.identify_palettes();

        // 3. Identify level data
        self.identify_level_data();

        // 4. Identify sprite metadata
        self.identify_sprite_data();

        eprintln!("  Found {} data segments", self.segments.len());
    }

    /// Find all LDA/LDX/LDY instructions that reference data
    fn find_data_references(&mut self) {
        for i in 0..self.rom.len() - 2 {
            let opcode = self.rom[i];

            // LDA absolute: 0xAD
            // LDX absolute: 0xAE
            // LDY absolute: 0xAC
            if matches!(opcode, 0xAD | 0xAE | 0xAC) {
                let addr = u16::from_le_bytes([
                    self.rom[i + 1],
                    self.rom[i + 2],
                ]);

                let code_addr = self.offset_to_addr(i);
                self.code_references
                    .entry(addr)
                    .or_insert_with(Vec::new)
                    .push(code_addr);
            }
        }

        eprintln!("  Found {} data references", self.code_references.len());
    }

    /// Identify palette data (32 bytes, specific pattern)
    fn identify_palettes(&mut self) {
        for i in 0..self.rom.len().saturating_sub(32) {
            if self.is_palette_pattern(&self.rom[i..i + 32]) {
                let addr = self.offset_to_addr(i);

                self.segments.push(DataSegment {
                    address: addr,
                    size: 32,
                    data_type: DataType::Palette,
                    data: self.rom[i..i + 32].to_vec(),
                    referenced_by: self.code_references
                        .get(&addr)
                        .cloned()
                        .unwrap_or_default(),
                });
            }
        }

        eprintln!("  Identified {} palettes",
            self.segments.iter().filter(|s| s.data_type == DataType::Palette).count());
    }

    /// Check if data looks like a palette
    fn is_palette_pattern(&self, data: &[u8]) -> bool {
        if data.len() != 32 {
            return false;
        }

        // Palettes have specific patterns:
        // 1. All values < 0x40 (NES palette index)
        // 2. Every 4th byte is often 0x0F or 0x00 (background color)
        // 3. No extreme variations

        let all_valid = data.iter().all(|&b| b < 0x40);
        if !all_valid {
            return false;
        }

        // Check for background color pattern
        let bg_pattern = (0..8).filter(|&i| {
            let val = data[i * 4];
            val == 0x0F || val == 0x00 || val == 0x20
        }).count();

        bg_pattern >= 4
    }

    /// Identify level data (more complex heuristics)
    fn identify_level_data(&mut self) {
        // Level data typically:
        // - Referenced frequently
        // - Has run-length encoding pattern
        // - Starts with specific headers

        for (&addr, refs) in &self.code_references {
            if refs.len() < 3 {
                continue; // Not referenced enough
            }

            let offset = self.addr_to_offset(addr);
            if offset + 100 > self.rom.len() {
                continue;
            }

            let data = &self.rom[offset..offset + 100];

            if self.is_level_data_pattern(data) {
                // Determine actual size by scanning
                let size = self.find_level_data_size(offset);

                self.segments.push(DataSegment {
                    address: addr,
                    size,
                    data_type: DataType::LevelData,
                    data: self.rom[offset..offset + size].to_vec(),
                    referenced_by: refs.clone(),
                });
            }
        }

        eprintln!("  Identified {} level data segments",
            self.segments.iter().filter(|s| s.data_type == DataType::LevelData).count());
    }

    /// Check if data looks like level data
    fn is_level_data_pattern(&self, data: &[u8]) -> bool {
        // Mario level data uses RLE and specific command bytes
        // Look for patterns like:
        // - Command bytes in specific ranges
        // - Repeated values (RLE)
        // - Length indicators

        let mut rle_count = 0;
        let mut command_count = 0;

        for i in 0..data.len() - 1 {
            // Check for RLE pattern (repeated bytes)
            if data[i] == data[i + 1] {
                rle_count += 1;
            }

            // Check for command-like bytes
            if data[i] >= 0x40 && data[i] < 0xF0 {
                command_count += 1;
            }
        }

        rle_count > 5 && command_count > 10
    }

    /// Find the size of level data by scanning for terminator
    fn find_level_data_size(&self, offset: usize) -> usize {
        // Level data typically ends with 0xFF or specific pattern
        for i in offset..self.rom.len().min(offset + 500) {
            if self.rom[i] == 0xFF {
                // Check if followed by more 0xFF (end marker)
                if i + 1 < self.rom.len() && self.rom[i + 1] == 0xFF {
                    return i - offset + 2;
                }
            }
        }

        100 // Default size
    }

    /// Identify sprite metadata
    fn identify_sprite_data(&mut self) {
        // Sprite data: groups of 4 bytes (Y, tile, attr, X)
        // Referenced by sprite initialization code

        for (&addr, refs) in &self.code_references {
            let offset = self.addr_to_offset(addr);
            if offset + 16 > self.rom.len() {
                continue;
            }

            let data = &self.rom[offset..offset + 16];

            if self.is_sprite_pattern(data) {
                let size = ((data.len() / 4) * 4).max(4);

                self.segments.push(DataSegment {
                    address: addr,
                    size,
                    data_type: DataType::SpriteMetadata,
                    data: self.rom[offset..offset + size].to_vec(),
                    referenced_by: refs.clone(),
                });
            }
        }

        eprintln!("  Identified {} sprite data segments",
            self.segments.iter().filter(|s| s.data_type == DataType::SpriteMetadata).count());
    }

    /// Check if data looks like sprite definitions
    fn is_sprite_pattern(&self, data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // Sprite data: Y, tile, attr, X
        // Y: 0-239
        // tile: any
        // attr: 0-3 (palette)
        // X: 0-255

        for chunk in data.chunks_exact(4) {
            let y = chunk[0];
            let _tile = chunk[1];
            let attr = chunk[2];
            let _x = chunk[3];

            if y > 239 || attr > 3 {
                return false;
            }
        }

        true
    }

    /// Get all identified segments
    pub fn segments(&self) -> &[DataSegment] {
        &self.segments
    }

    /// Generate NESL code for data segments
    pub fn generate_nesl(&self) -> String {
        let mut code = String::new();

        code.push_str("// ========== Data Segments ==========\n\n");

        // Palettes
        for (i, seg) in self.segments.iter()
            .filter(|s| s.data_type == DataType::Palette)
            .enumerate()
        {
            code.push_str(&format!("// Palette at ${:04X}\n", seg.address));
            code.push_str(&format!("const palette_{}: [u8] = [\n", i));

            for (j, chunk) in seg.data.chunks(4).enumerate() {
                code.push_str("    ");
                for &byte in chunk {
                    code.push_str(&format!("0x{:02X}, ", byte));
                }
                code.push_str(&format!(" // {}\n", match j {
                    0 | 4 => "Background/Sprite color 0",
                    _ => "Colors 1-3",
                }));
            }

            code.push_str("];\n\n");
        }

        // Level data
        for (i, seg) in self.segments.iter()
            .filter(|s| s.data_type == DataType::LevelData)
            .enumerate()
        {
            code.push_str(&format!("// Level data at ${:04X}\n", seg.address));
            code.push_str(&format!("const level_{}: [u8] = [\n", i));

            for chunk in seg.data.chunks(16) {
                code.push_str("    ");
                for &byte in chunk {
                    code.push_str(&format!("0x{:02X}, ", byte));
                }
                code.push_str("\n");
            }

            code.push_str("];\n\n");
        }

        code
    }

    fn addr_to_offset(&self, addr: u16) -> usize {
        if addr >= 0xC000 && self.rom.len() <= 16384 {
            (addr - 0xC000) as usize
        } else if addr >= 0x8000 {
            (addr - 0x8000) as usize
        } else {
            0
        }
    }

    fn offset_to_addr(&self, offset: usize) -> u16 {
        if self.rom.len() <= 16384 {
            0xC000 + offset as u16
        } else {
            0x8000 + offset as u16
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_detection() {
        let analyzer = DataAnalyzer::new(&[]);

        // Valid palette
        let palette = vec![
            0x0F, 0x30, 0x10, 0x00,
            0x0F, 0x30, 0x27, 0x17,
            0x0F, 0x30, 0x16, 0x27,
            0x0F, 0x30, 0x02, 0x38,
            0x0F, 0x30, 0x10, 0x00,
            0x0F, 0x30, 0x27, 0x17,
            0x0F, 0x30, 0x16, 0x27,
            0x0F, 0x30, 0x02, 0x38,
        ];

        assert!(analyzer.is_palette_pattern(&palette));
    }
}

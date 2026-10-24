//! CHR-ROM tile decoding back to pixel strings.

/// Decodes a 16-byte NES tile (2bpp format) back to 8 rows of 8 pixel characters.
/// Returns pixel values 0-3 as chars: '.' (0), '1' (1), '2' (2), '3' (3).
pub fn decode_tile(data: &[u8; 16]) -> [String; 8] {
    let mut rows: [String; 8] = Default::default();

    for y in 0..8 {
        let mut row = String::with_capacity(8);
        let plane0 = data[y];
        let plane1 = data[y + 8];

        for x in 0..8 {
            let bit = 0x80 >> x;
            let bit0 = if plane0 & bit != 0 { 1 } else { 0 };
            let bit1 = if plane1 & bit != 0 { 2 } else { 0 };
            let pixel = bit0 | bit1;

            row.push(match pixel {
                0 => '.',
                1 => '1',
                2 => '2',
                3 => '3',
                _ => unreachable!(),
            });
        }
        rows[y] = row;
    }

    rows
}

/// Whether `tile_data` is what the compiler puts at `tile_index` when no `tile` overrides
/// it: the built-in font (ASCII, lowercase reusing uppercase), status-bar markers, or blank.
pub fn is_builtin_tile(tile_index: u8, tile_data: &[u8; 16]) -> bool {
    let default = crate::chr::build_chr(&[]);
    let at = tile_index as usize * 16;
    default[at..at + 16] == tile_data[..]
}

/// Scans CHR-ROM and extracts user-defined tiles (non-font tiles).
pub fn extract_user_tiles(chr_rom: &[u8]) -> Vec<(u8, [String; 8])> {
    let mut tiles = Vec::new();

    // Scan all 256 tiles in pattern table 0
    for tile_index in 0..256u16 {
        let offset = (tile_index * 16) as usize;
        if offset + 16 > chr_rom.len() {
            break;
        }

        let tile_data: [u8; 16] = chr_rom[offset..offset + 16].try_into().unwrap();

        if !is_builtin_tile(tile_index as u8, &tile_data) {
            let pattern = decode_tile(&tile_data);
            tiles.push((tile_index as u8, pattern));
        }
    }

    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty_tile() {
        let tile = [0u8; 16];
        let rows = decode_tile(&tile);
        for row in &rows {
            assert_eq!(row, "........");
        }
    }

    #[test]
    fn decode_simple_pattern() {
        // Create a simple X pattern: pixels at (0,0), (7,0), (0,7), (7,7)
        let mut tile = [0u8; 16];
        tile[0] = 0x81; // Top row: bits 7 and 0 set
        tile[7] = 0x81; // Bottom row: bits 7 and 0 set

        let rows = decode_tile(&tile);
        assert_eq!(rows[0], "1......1");
        assert_eq!(rows[1], "........");
        assert_eq!(rows[7], "1......1");
    }

    #[test]
    fn roundtrip_encode_decode() {
        use crate::chr::encode_tile;

        let original = [
            "........",
            ".1111...",
            ".1...1..",
            ".1111...",
            ".1......",
            ".1......",
            "........",
            "........",
        ];

        let encoded = encode_tile(&original).unwrap();
        let decoded = decode_tile(&encoded);

        for (i, (orig, dec)) in original.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig, dec, "Row {} mismatch", i);
        }
    }
}

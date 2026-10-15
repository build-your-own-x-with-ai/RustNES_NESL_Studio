//! CHR (pattern table) generation: built-in font plus user `tile` definitions.

pub fn encode_tile(_rows: &[&str]) -> Result<[u8; 16], String> {
    Err("not implemented".into())
}

pub fn build_chr(_tiles: &[(u8, [u8; 16])]) -> Vec<u8> {
    vec![0; crate::rom::CHR_SIZE]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_two_bitplanes_msb_first() {
        let t = encode_tile(&["0123....", "3.......", "........", "........", "........", "........", "........", "......21"]).unwrap();
        assert_eq!(t[0], 0b0101_0000);
        assert_eq!(t[8], 0b0011_0000);
        assert_eq!((t[1], t[9]), (0x80, 0x80));
        assert_eq!((t[7], t[15]), (0b0000_0001, 0b0000_0010));
    }

    #[test]
    fn rejects_bad_shapes_and_pixels() {
        assert!(encode_tile(&["........"; 7]).unwrap_err().contains("8 rows"));
        let mut rows = ["........"; 8];
        rows[2] = ".......";
        assert!(encode_tile(&rows).unwrap_err().contains("row 3"));
        rows[2] = "...4....";
        assert!(encode_tile(&rows).unwrap_err().contains("pixel"));
    }

    #[test]
    fn font_covers_printable_ascii_and_lowercase_maps_to_uppercase() {
        let chr = build_chr(&[]);
        assert!(chr[0x20 * 16..0x21 * 16].iter().all(|b| *b == 0), "space is blank");
        for c in b'!'..=b'_' {
            let g = &chr[c as usize * 16..c as usize * 16 + 8];
            assert!(g.iter().any(|b| *b != 0), "glyph {:?}", c as char);
        }
        assert_eq!(&chr[b'a' as usize * 16..b'a' as usize * 16 + 16], &chr[b'A' as usize * 16..b'A' as usize * 16 + 16]);
    }

    #[test]
    fn user_tiles_override_font() {
        let chr = build_chr(&[(0x41, [0xAA; 16])]);
        assert_eq!(&chr[0x410..0x420], &[0xAA; 16]);
        assert_eq!(&chr[0x1410..0x1420], &[0xAA; 16]);
    }
}

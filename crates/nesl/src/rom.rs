//! iNES (NROM-256) image packing.

pub const PRG_SIZE: usize = 0x8000;
pub const CHR_SIZE: usize = 0x2000;
pub const HEADER_SIZE: usize = 16;

/// Lays out `code` (assembled at $8000) into a 32KB PRG bank with the three
/// interrupt vectors at $FFFA, and wraps it with `chr` into an iNES file.
pub fn build_ines(code: &[u8], nmi: u16, reset: u16, irq: u16, chr: &[u8]) -> Result<Vec<u8>, String> {
    let limit = PRG_SIZE - 6;
    if code.len() > limit {
        return Err(format!("program is too large: {} bytes of code/data, limit is {limit}", code.len()));
    }
    if chr.len() != CHR_SIZE {
        return Err(format!("CHR data must be {CHR_SIZE} bytes, got {}", chr.len()));
    }
    let mut rom = Vec::with_capacity(HEADER_SIZE + PRG_SIZE + CHR_SIZE);
    // "NES\x1A", 2x16KB PRG, 1x8KB CHR, flags6: mapper 0 + vertical mirroring
    rom.extend_from_slice(&[b'N', b'E', b'S', 0x1A, 2, 1, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    rom.extend_from_slice(code);
    rom.resize(HEADER_SIZE + limit, 0xFF);
    for v in [nmi, reset, irq] {
        rom.extend_from_slice(&v.to_le_bytes());
    }
    rom.extend_from_slice(chr);
    Ok(rom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_layout_and_vectors() {
        let chr = vec![0x11; CHR_SIZE];
        let rom = build_ines(&[0xEA, 0x60], 0x8001, 0x8000, 0x8002, &chr).unwrap();
        assert_eq!(rom.len(), 16 + 32768 + 8192);
        assert_eq!(&rom[..8], &[b'N', b'E', b'S', 0x1A, 2, 1, 0x01, 0]);
        assert_eq!(&rom[16..18], &[0xEA, 0x60]);
        assert_eq!(rom[18], 0xFF, "unused PRG is padded with $FF");
        let v = 16 + 0x7FFA;
        assert_eq!(&rom[v..v + 6], &[0x01, 0x80, 0x00, 0x80, 0x02, 0x80]);
        assert_eq!(rom[16 + 32768], 0x11);
    }

    #[test]
    fn rejects_oversized_programs_and_bad_chr() {
        assert!(build_ines(&vec![0; PRG_SIZE], 0, 0, 0, &[0; CHR_SIZE]).unwrap_err().contains("too large"));
        assert!(build_ines(&[], 0, 0, 0, &[0; 10]).is_err());
    }
}

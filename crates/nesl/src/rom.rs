//! iNES (NROM-256) image packing.

pub const PRG_SIZE: usize = 0x8000;
pub const CHR_SIZE: usize = 0x2000;
pub const HEADER_SIZE: usize = 16;

/// Lays out `code` (assembled at $8000) into a 32KB PRG bank with the three
/// interrupt vectors at $FFFA, and wraps it with `chr` into an iNES file.
/// Builds an iNES ROM for UNROM (mapper 2) with multiple PRG banks and CHR-RAM.
/// `banks` contains switchable 16KB banks; `fixed` is the last 16KB bank.
pub fn build_unrom(banks: Vec<Vec<u8>>, fixed: &[u8], nmi: u16, reset: u16, irq: u16) -> Result<Vec<u8>, String> {
    if banks.is_empty() || banks.len() > 16 {
        return Err(format!("UNROM requires 1-16 banks, got {}", banks.len()));
    }
    if fixed.len() > 0x4000 {
        return Err(format!("fixed bank too large: {} bytes (max 16384)", fixed.len()));
    }
    
    // Round bank count to nearest power of 2
    let bank_count = banks.len() + 1; // +1 for fixed
    let prg_banks = bank_count.next_power_of_two().max(2);
    
    let mut rom = vec![
        b'N', b'E', b'S', 0x1A,
        prg_banks as u8,
        0, // CHR count = 0 (CHR-RAM)
        0x23, // flags6: mapper 2 low nibble, bit 0 = vertical mirroring, bit 1 = battery (SRAM)
        0x00, // flags7: mapper 2 high nibble
    ];
    rom.resize(16, 0);
    
    // Switchable banks
    for bank in &banks {
        if bank.len() > 0x4000 {
            return Err(format!("bank too large: {} bytes", bank.len()));
        }
        rom.extend_from_slice(bank);
        rom.resize(rom.len() + (0x4000 - bank.len()), 0); // pad to 16KB
    }
    
    // Pad with empty switchable banks so the fixed bank is the last of a power of two
    for _ in bank_count..prg_banks {
        rom.resize(rom.len() + 0x4000, 0);
    }

    // Fixed bank (must be last: the CPU always sees it at $C000-$FFFF)
    let mut fixed_bank = fixed.to_vec();
    fixed_bank.resize(0x4000, 0);
    let vec_offset = 0x4000 - 6;
    fixed_bank[vec_offset..].copy_from_slice(&[
        nmi as u8, (nmi >> 8) as u8, reset as u8, (reset >> 8) as u8, irq as u8, (irq >> 8) as u8,
    ]);
    rom.extend_from_slice(&fixed_bank);

    Ok(rom)
}

pub fn build_ines(code: &[u8], nmi: u16, reset: u16, irq: u16, chr: &[u8]) -> Result<Vec<u8>, String> {
    let limit = PRG_SIZE - 6;
    if code.len() > limit {
        return Err(format!("program is too large: {} bytes of code/data, limit is {limit}", code.len()));
    }
    if chr.len() != CHR_SIZE {
        return Err(format!("CHR data must be {CHR_SIZE} bytes, got {}", chr.len()));
    }
    let mut rom = Vec::with_capacity(HEADER_SIZE + PRG_SIZE + CHR_SIZE);
    // "NES\x1A", 2x16KB PRG, 1x8KB CHR, flags6: mapper 0 + vertical mirroring + battery (SRAM)
    rom.extend_from_slice(&[b'N', b'E', b'S', 0x1A, 2, 1, 0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    rom.extend_from_slice(code);
    rom.resize(HEADER_SIZE + limit, 0xFF);
    for v in [nmi, reset, irq] {
        rom.extend_from_slice(&v.to_le_bytes());
    }
    rom.extend_from_slice(chr);
    Ok(rom)
}

/// Start of the MMC3 reset stub in the fixed bank; program code must end before it.
pub const MMC3_STUB: u16 = 0xFFD0;

/// Builds an MMC3 (mapper 4) image with 32KB PRG mapped linearly ($8000-$FFFF) and the
/// 8KB `chr` (tiles in $0000-$0FFF) copied into both pattern tables. A stub at
/// `MMC3_STUB` sets the bank registers (undefined at power-on) before jumping to `reset`.
pub fn build_mmc3(code: &[u8], nmi: u16, reset: u16, irq: u16, chr: &[u8]) -> Result<Vec<u8>, String> {
    let limit = (MMC3_STUB - 0x8000) as usize;
    if code.len() > limit {
        return Err(format!("program is too large for MMC3 (32KB): {} bytes of code/data, limit is {limit}", code.len()));
    }
    // flags6: mapper 4 low nibble, battery, vertical mirroring
    let mut rom = vec![b'N', b'E', b'S', 0x1A, 2, 1, 0x43, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    rom.extend_from_slice(code);
    rom.resize(HEADER_SIZE + PRG_SIZE, 0xFF);
    #[rustfmt::skip]
    let mut stub = vec![
        0x78,                     // SEI
        0xA2, 0x07,               // LDX #7
        0x8E, 0x00, 0x80,         // loop: STX $8000  select R[X]
        0xBD, 0, 0,               // LDA table,X
        0x8D, 0x01, 0x80,         // STA $8001
        0xCA,                     // DEX
        0x10, 0xF4,               // BPL loop
        0xA9, 0x00, 0x8D, 0x00, 0xA0, // vertical mirroring
        0xA9, 0x80, 0x8D, 0x01, 0xA0, // PRG-RAM enabled
        0x4C, reset as u8, (reset >> 8) as u8,
    ];
    let table = MMC3_STUB + stub.len() as u16;
    stub[7] = table as u8;
    stub[8] = (table >> 8) as u8;
    // R0/R1: 2KB CHR at $0000/$0800; R2-R5: 1KB at $1000-$1C00; R6/R7: PRG 0/1
    stub.extend_from_slice(&[0, 2, 4, 5, 6, 7, 0, 1]);
    let at = HEADER_SIZE + (MMC3_STUB - 0x8000) as usize;
    rom[at..at + stub.len()].copy_from_slice(&stub);
    let v = HEADER_SIZE + PRG_SIZE - 6;
    for (i, a) in [nmi, MMC3_STUB, irq].iter().enumerate() {
        rom[v + 2 * i..v + 2 * i + 2].copy_from_slice(&a.to_le_bytes());
    }
    if chr.len() != CHR_SIZE {
        return Err(format!("CHR data must be {CHR_SIZE} bytes, got {}", chr.len()));
    }
    rom.extend_from_slice(&chr[..0x1000]);
    rom.extend_from_slice(&chr[..0x1000]);
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
        assert_eq!(&rom[..8], &[b'N', b'E', b'S', 0x1A, 2, 1, 0x03, 0]);
        assert_eq!(&rom[16..18], &[0xEA, 0x60]);
        assert_eq!(rom[18], 0xFF, "unused PRG is padded with $FF");
        let v = 16 + 0x7FFA;
        assert_eq!(&rom[v..v + 6], &[0x01, 0x80, 0x00, 0x80, 0x02, 0x80]);
        assert_eq!(rom[16 + 32768], 0x11);
    }

    #[test]
    fn rejects_oversized_programs_and_badchr() {
        assert!(build_ines(&vec![0; PRG_SIZE], 0, 0, 0, &[0; CHR_SIZE]).unwrap_err().contains("too large"));
        assert!(build_ines(&[], 0, 0, 0, &[0; 10]).is_err());
    }
}

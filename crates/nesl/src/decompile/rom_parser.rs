//! ROM image parsing and structure extraction.

use crate::decompile::DecompileError;
use std::path::Path;
use std::fs;

/// Parsed iNES header information.
#[derive(Debug, Clone)]
pub struct InesHeader {
    pub prg_banks: u8,
    pub chr_banks: u8,
    pub mapper: u8,
    pub mirroring: u8,
}

/// Parsed ROM image with separated components.
#[derive(Debug, Clone)]
pub struct RomImage {
    pub header: InesHeader,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

/// Loads and parses an iNES ROM file.
pub fn load_rom(path: &Path) -> Result<RomImage, DecompileError> {
    let data = fs::read(path).map_err(|_| DecompileError::RomNotFound {
        path: path.display().to_string(),
    })?;

    if data.len() < 16 {
        return Err(DecompileError::InvalidRom {
            reason: "文件太小，不是有效的 iNES ROM".into(),
        });
    }

    // Check magic number
    if &data[0..4] != b"NES\x1a" {
        return Err(DecompileError::InvalidRom {
            reason: "缺少 iNES 魔数 (NES\\x1a)".into(),
        });
    }

    let prg_banks = data[4];
    let chr_banks = data[5];
    let flags6 = data[6];
    let flags7 = data[7];

    let mapper = (flags7 & 0xF0) | (flags6 >> 4);
    let mirroring = flags6 & 0x01;

    let prg_size = prg_banks as usize * 16384;
    let chr_size = chr_banks as usize * 8192;

    if data.len() < 16 + prg_size + chr_size {
        return Err(DecompileError::InvalidRom {
            reason: format!("ROM 大小不匹配：期望 {} 字节，实际 {} 字节",
                16 + prg_size + chr_size, data.len()),
        });
    }

    let prg_rom = data[16..16 + prg_size].to_vec();
    let chr_rom = data[16 + prg_size..16 + prg_size + chr_size].to_vec();

    Ok(RomImage {
        header: InesHeader {
            prg_banks,
            chr_banks,
            mapper,
            mirroring,
        },
        prg_rom,
        chr_rom,
    })
}

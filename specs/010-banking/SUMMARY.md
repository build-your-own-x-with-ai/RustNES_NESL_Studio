# 010: Automatic UNROM Banking

**Status**: ✅ Complete  
**Commits**: [010-banking branch](https://github.com/*/RustNES/tree/010-banking)

## What Changed

Programs under 32KB stay NROM (mapper 0). Larger programs automatically switch to UNROM (mapper 2):

- Maps split across up to 16 switchable 16KB banks
- Code runs from a fixed 16KB bank at $C000-$FFFF  
- 8KB CHR data copied into CHR-RAM at reset
- Map builtins switch banks before accessing map data (bus-conflict safe)

COIN QUEST grew from 3 worlds (~12KB) to 6 worlds (~34KB), compiling to UNROM with zero code changes beyond map definitions.

## Test Results

- All 276 workspace tests pass
- New tests: UNROM triggering, CHR copy, bank padding, 6-world progression, layout overlap detection
- jsnes cross-check: UNROM ROMs run correctly under mapper 2
- FR-1001: all 7 existing example ROMs byte-identical to pre-banking builds

## Jump Physics Fix

Platforms were unreachable with the original jump height. Changed `JUMP` from -7 to -9, giving 36px rise:
- Ground = row 21
- Tier A platforms = row 17 (one jump from ground)
- Tier B platforms = row 13 (one jump from tier A)
- Pits capped at 4 tiles (28px) to stay within horizontal clearance

## Known Limitations

- Max 16 banks (240KB map data + 16KB code)
- CHR-RAM only (no CHR-ROM mixing)
- Bank switching adds ~5 cycles per map operation
- Requires power-of-2 PRG bank count (ROM padded with empty banks)

## Files Changed

- `crates/nesl/src/{lib,codegen,rom,runtime,builtins}.rs`: two-pass compilation, bank assignment, UNROM builder, __bank/__chr_copy
- `crates/nes-sim/src/bus.rs`: mapper 2 implementation, bus-conflict tracking
- `examples/quest.nesl`: 6 worlds, JUMP=-9
- `crates/nesl/tests/{banking,quest}.rs`: UNROM tests, layout overlap test
- `specs/010-banking/{SPEC,IMPLEMENTATION,SUMMARY}.md`: documentation

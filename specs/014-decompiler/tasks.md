# Tasks: NESL Decompiler

**Input**: Design documents from `/specs/014-decompiler/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: TDD 强制执行（宪章 Principle II）—— 每个能力先写失败测试

**Organization**: 按用户故事（User Story）组织任务，每个故事独立可测

---

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行执行（不同文件，无依赖）
- **[Story]**: 任务所属用户故事（US1, US2, US3, US4）
- 包含准确的文件路径

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 项目初始化和基础结构

- [X] T001 Add `serde_json` dependency to `crates/nesl/Cargo.toml` (version = "1.0")
- [X] T002 [P] Add `serde` dependency to `crates/nesl/Cargo.toml` with derive feature (version = "1.0")
- [X] T003 Create `crates/nesl/src/decompile/` directory with `mod.rs`
- [X] T004 Create `crates/nesl/src/debug_symbols.rs` for symbol table generation
- [X] T005 Create `crates/nesl/tests/decompile.rs` test file
- [X] T006 Create `crates/nesl/tests/roundtrip.rs` test file
- [X] T007 [P] Create `crates/nesc/src/cmd/decompile.rs` CLI command file
- [X] T008 Add `decompile` subcommand to `crates/nesc/src/main.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 核心数据结构和符号表格式，所有用户故事依赖

**⚠️ CRITICAL**: 所有用户故事工作必须在此阶段完成后才能开始

### Data Structures (from data-model.md)

- [X] T009 [P] Define `SymbolTable` struct in `crates/nesl/src/debug_symbols.rs` with fields: version (String), compiler_version (String), mapper (u8), prg_size (usize), chr_size (usize), variables (Vec<Variable>), constants (Vec<Constant>), functions (Vec<Function>), resources (Vec<Resource>), builtins (Vec<Builtin>), source_map (Vec<SourceMapEntry>)
- [X] T010 [P] Define `Variable` struct with fields: name (String), ty (Type), addr (u16), init_value (Option<Value>) in `crates/nesl/src/debug_symbols.rs`
- [X] T011 [P] Define `Type` enum (U8, U16, I8, Array, Struct) in `crates/nesl/src/debug_symbols.rs`
- [X] T012 [P] Define `Constant` struct with fields: name (String), ty (Type), addr (u16), data (Vec<u8>) in `crates/nesl/src/debug_symbols.rs`
- [X] T013 [P] Define `Function` struct with fields: name (String), addr (u16), params (Vec<(String, Type)>), return_ty (Option<Type>), local_vars (Vec<Variable>) in `crates/nesl/src/debug_symbols.rs`
- [X] T014 [P] Define `Resource` struct with fields: name (String), resource_ty (ResourceType), location (ResourceLocation) in `crates/nesl/src/debug_symbols.rs`
- [X] T015 [P] Define `ResourceType` enum (Tile, Map, Music, Metasprite, Sound) in `crates/nesl/src/debug_symbols.rs`
- [X] T016 [P] Define `ResourceLocation` enum (ChrRom, PrgRom, Bank) in `crates/nesl/src/debug_symbols.rs`
- [X] T017 [P] Define `Builtin` struct with fields: name (String), addr (u16), signature (String) in `crates/nesl/src/debug_symbols.rs`
- [X] T018 [P] Define `SourceMapEntry` struct with fields: nes_addr (u16), source_line (usize), source_file (String) in `crates/nesl/src/debug_symbols.rs`
- [X] T019 [P] Define `Value` enum (U8, U16, I8, Array) in `crates/nesl/src/debug_symbols.rs`
- [X] T020 Add `#[derive(Serialize, Deserialize)]` to all data structures in `crates/nesl/src/debug_symbols.rs`

### Symbol Table Generation (contracts/symbol-table.md)

- [X] T021 Implement `generate_symbols(out: &CompileOutput) -> SymbolTable` in `crates/nesl/src/debug_symbols.rs` returning stub SymbolTable with version "1.0.0"
- [X] T022 Add `--debug-symbols` flag to `crates/nesc/src/cmd/build.rs` that calls `generate_symbols()` and writes `.syms` JSON file
- [X] T023 Implement `parse_symbols(json: &str) -> Result<SymbolTable, DecompileError>` in `crates/nesl/src/decompile/mod.rs` using `serde_json::from_str`

### Error Types (data-model.md)

- [X] T024 [P] Define `DecompileError` enum in `crates/nesl/src/decompile/mod.rs` with variants: RomNotFound { path: String }, SymbolsNotFound { path: String }, InvalidRom { reason: String }, InvalidSymbols { reason: String }, UnsupportedVersion { found: String, supported: String }, UnsupportedMapper { mapper: u8 }, SymbolMismatch { reason: String }, NotNeslRom, UnrecognizedPattern { addr: u16, desc: String }, Io(std::io::Error)
- [X] T025 Implement `Display` for `DecompileError` in `crates/nesl/src/decompile/mod.rs` with clear, actionable error messages per contracts/decompile-api.md
- [X] T026 Implement `Error` for `DecompileError` in `crates/nesl/src/decompile/mod.rs`
- [X] T027 Implement `From<std::io::Error>` for `DecompileError` in `crates/nesl/src/decompile/mod.rs`

### ROM Parser

- [X] T028 Create `crates/nesl/src/decompile/rom_parser.rs` with `RomImage` struct containing header (InesHeader), prg_rom (Vec<u8>), chr_rom (Vec<u8>)
- [X] T029 Implement `load_rom(path: &Path) -> Result<RomImage, DecompileError>` in `crates/nesl/src/decompile/rom_parser.rs` that validates iNES header (NES\x1A magic), extracts mapper, prg_size, chr_size

**Checkpoint**: Foundation ready — 用户故事实现可以并行开始

---

## Phase 3: User Story 1 - Round-trip Compilation (Priority: P1) 🎯 MVP

**Goal**: 简单程序（hello.nesl）编译 → 反编译 → 重新编译，验证往返一致性

**Independent Test**: 使用 examples/hello.nesl 往返测试，断言 ROM 语义等价

### Tests for User Story 1 (TDD - Write FIRST, ensure they FAIL) ⚠️

- [X] T030 [P] [US1] Write failing test `roundtrip_hello_basic` in `crates/nesl/tests/roundtrip.rs`: compile examples/hello.nesl → generate stub symbols → decompile (should fail: not implemented) → assert decompiled code is valid NESL
- [X] T031 [P] [US1] Write failing test `symbols_generation_hello` in `crates/nesl/tests/decompile.rs`: compile examples/hello.nesl → generate symbols → assert symbols.variables contains "x" with ty=u8, addr>0, init_value=Some(120)
- [X] T032 [P] [US1] Write failing test `parse_generated_symbols` in `crates/nesl/tests/decompile.rs`: generate symbols JSON → parse_symbols() → assert SymbolTable fields match

### Symbol Table Generation for US1

- [X] T033 [US1] Implement variable extraction in `generate_symbols()` in `crates/nesl/src/debug_symbols.rs`: iterate `out.vars`, populate `SymbolTable.variables` with name, type (from codegen), addr, init_value (from CompileOutput)
- [X] T034 [US1] Implement constant extraction in `generate_symbols()`: iterate `out.constants`, populate `SymbolTable.constants` with name, ty, addr, data (read from out.rom at addr)
- [X] T035 [US1] Implement function extraction in `generate_symbols()`: iterate `out.functions`, populate `SymbolTable.functions` with name, addr, params (from parser AST), return_ty, local_vars (empty for now)
- [X] T036 [US1] Implement builtin mapping in `generate_symbols()`: hardcode runtime addresses from crates/nesl/src/runtime.rs (e.g. __sprite: 0xC000), populate `SymbolTable.builtins` with name, addr, signature (from builtins.rs)
- [X] T037 [US1] Add `mapper`, `prg_size`, `chr_size` fields to `generate_symbols()` from `out.mapper`, `out.rom.len()`, `out.chr.len()`
- [X] T038 [US1] Write symbols JSON to `.syms` file in `crates/nesc/src/cmd/build.rs` when `--debug-symbols` flag present: call `serde_json::to_string_pretty(&symbols)`, write to `<output>.syms`

### Decompile Core for US1

- [X] T039 [US1] Create `crates/nesl/src/decompile/symbol_loader.rs` with `load_symbols(syms_path: &Path) -> Result<SymbolTable, DecompileError>` that reads file, calls `parse_symbols()`
- [X] T040 [US1] Implement `decompile(rom_path: &Path, symbols_path: Option<&Path>) -> Result<String, DecompileError>` in `crates/nesl/src/decompile/mod.rs`: discover `.syms` if None, load ROM, load symbols, validate consistency (mapper, prg_size match), return stub output "// Decompiled from <rom>\n"
- [X] T041 [US1] Implement symbol table validation in `decompile()`: check symbols.version starts with "1.", symbols.mapper matches ROM header mapper, symbols.prg_size/chr_size match ROM, return SymbolMismatch error if mismatch
- [X] T042 [US1] Create `crates/nesl/src/decompile/ast_builder.rs` with `build_ast(rom: &RomImage, symbols: &SymbolTable) -> Vec<TopLevel>` returning empty Vec for now
- [X] T043 [US1] Implement variable restoration in `build_ast()`: iterate `symbols.variables`, create `TopLevel::Var { name, ty, init: Some(Expr::Literal(init_value)) }` for each
- [X] T044 [US1] Implement constant restoration in `build_ast()`: iterate `symbols.constants`, create `TopLevel::Const { name, ty, value: ConstValue::Array(data) }` for each
- [X] T045 [US1] Implement function signature restoration in `build_ast()`: iterate `symbols.functions`, create `TopLevel::Function { name, params, return_ty, body: Block { stmts: vec![] } }` (empty body for now)
- [X] T046 [US1] Create `crates/nesl/src/decompile/codegen.rs` with `generate_source(ast: &[TopLevel]) -> String` that formats TopLevel::Var as "var name: ty = init_value;", TopLevel::Const as "const name: [ty] = [...];", TopLevel::Function as "fn name(params) -> return_ty {}"
- [X] T047 [US1] Add header comment in `generate_source()`: prepend "// Decompiled from <rom>\n// NESL Compiler: <compiler_version>\n// Decompiled at: <timestamp>\n\n" to output
- [X] T048 [US1] Connect components in `decompile()`: call `build_ast(rom, symbols)`, call `generate_source(&ast)`, return result

### CLI Integration for US1

- [X] T049 [US1] Implement `nesc decompile` command in `crates/nesc/src/cmd/decompile.rs`: parse args (rom_path, --symbols, -o), call `nesl::decompile::decompile()`, write output to file or stdout
- [X] T050 [US1] Add error handling in CLI: match `DecompileError` variants, print user-friendly messages ("找不到符号表文件: <path>\n提示：使用 `nesc build --debug-symbols` 生成符号表"), exit with appropriate code (1-5 per contracts/decompile-api.md)
- [X] T051 [US1] Add `--validate` flag to `nesc decompile`: only validate symbols vs ROM, print "✓ 符号表有效" and exit 0, or error and exit 3-4

### Validation for US1

- [X] T052 [US1] Run test `roundtrip_hello_basic`: compile hello.nesl with --debug-symbols → decompile → parse decompiled code with nesl::parser → assert AST contains main function, variable x
- [X] T053 [US1] Run test `symbols_generation_hello`: assert symbols.variables[0].name == "x", symbols.constants[0].name == "PAL", symbols.functions[0].name == "main"
- [X] T054 [US1] Manual validation: run `nesc build examples/hello.nesl --debug-symbols && nesc decompile examples/hello.nes -o /tmp/hello.dec.nesl && cat /tmp/hello.dec.nesl`, verify output contains `var x: u8 = 124;`, `const PAL: [u8] = [...]`, `fn main() {}`

**Checkpoint**: User Story 1 完成 — 基本往返流程可用，符号表生成和反编译核心工作

---

## Phase 4: User Story 2 - NROM Resource Reconstruction (Priority: P2)

**Goal**: 还原 tile/map/palette 资源定义

**Independent Test**: 使用 examples/sprite.nesl 或 examples/game.nesl，检查反编译输出包含完整的 tile 和调色板

### Tests for User Story 2 (TDD) ⚠️

- [ ] T055 [P] [US2] Write failing test `tile_restoration` in `crates/nesl/tests/decompile.rs`: compile program with `tile 1 = [...]` → generate symbols → decompile → assert output contains "tile 1 = [" and 8 lines of pixel strings
- [ ] T056 [P] [US2] Write failing test `map_restoration` in `crates/nesl/tests/decompile.rs`: compile program with `map LEVEL = [...]` → decompile → assert output contains "map LEVEL = [" and row strings
- [ ] T057 [P] [US2] Write failing test `roundtrip_sprite_example` in `crates/nesl/tests/roundtrip.rs`: compile examples/sprite.nesl → decompile → recompile → run both ROMs in nes-sim for 30 frames → assert VRAM[0x10..0x20] (tile 1 CHR data) identical

### Symbol Table for Resources (US2)

- [ ] T058 [US2] Add tile extraction to `generate_symbols()` in `crates/nesl/src/debug_symbols.rs`: for each tile definition in `out.tiles`, create `Resource { name: "tile_N", resource_ty: Tile, location: ChrRom { offset: N*16, length: 16 } }`, append to `symbols.resources`
- [ ] T059 [US2] Add map extraction to `generate_symbols()`: for each map in `out.maps`, create `Resource { name: map_label, resource_ty: Map { width, height }, location: PrgRom { addr: map_data_addr, length: width*height } }` or `Bank { bank, addr, length }` if UNROM, append to `symbols.resources`
- [ ] T060 [US2] Add palette extraction to `generate_symbols()`: if `out.constants` contains "PAL" with data.len()==32, mark it as special constant (no extra Resource entry, handled by Constant restoration)

### Resource Restoration in AST Builder (US2)

- [ ] T061 [US2] Implement tile restoration in `build_ast()` in `crates/nesl/src/decompile/ast_builder.rs`: for each Resource with resource_ty=Tile, read 16 bytes from `rom.chr_rom[location.offset..offset+16]`, convert to 8x8 pixel matrix (2 bitplanes), convert pixels (0='.', 1='1', 2='2', 3='3') to 8 strings, create `TopLevel::Tile { index: extract_N_from_name, pixels: [[u8; 8]; 8] }`
- [ ] T062 [US2] Implement map restoration in `build_ast()`: for each Resource with resource_ty=Map, read bytes from `rom.prg_rom[location.addr - 0x8000..]` (adjust for NROM base), convert bytes to tile characters (per map encoding in codegen.rs), group into `height` rows of `width` chars, create `TopLevel::Map { name, width, rows: Vec<String> }`
- [ ] T063 [US2] Handle missing CHR data: if `location.offset + length > rom.chr_rom.len()`, push warning to context.errors, create comment `TopLevel::Comment("// WARNING: tile N CHR data missing")`

### Code Generation for Resources (US2)

- [ ] T064 [US2] Implement tile formatting in `generate_source()` in `crates/nesl/src/decompile/codegen.rs`: for `TopLevel::Tile { index, pixels }`, output `tile <index> = [\n  "<row0>",\n  "<row1>",\n  ...\n  "<row7>"\n];` with each row as 8-char string
- [ ] T065 [US2] Implement map formatting in `generate_source()`: for `TopLevel::Map { name, width, rows }`, output `map <name> = [\n  "<row0>",\n  "<row1>",\n  ...\n];` with proper escaping for special chars
- [ ] T066 [US2] Add resource section ordering in `generate_source()`: sort TopLevel items as: Const → Var → Tile → Map → Music → Function, per NESL convention

### Validation for US2

- [ ] T067 [US2] Run test `tile_restoration`: assert decompiled output contains exactly "tile 1 = [" followed by 8 lines, first line starts with "..", verify pixel-to-char conversion correct
- [ ] T068 [US2] Run test `map_restoration`: assert map dimensions (width x height) match original, row count correct
- [ ] T069 [US2] Run test `roundtrip_sprite_example`: assert VRAM tile data identical, assert OAM sprite positions match after 30 frames
- [ ] T070 [US2] Manual validation: run `nesc build examples/game.nesl --debug-symbols && nesc decompile examples/game.nes -o /tmp/game.dec.nesl`, check output has all tiles and map LEVEL

**Checkpoint**: User Story 2 完成 — 资源还原工作，NROM 游戏可完整反编译

---

## Phase 5: User Story 3 - UNROM Multi-Bank Decompilation (Priority: P3)

**Goal**: 还原 UNROM (mapper 2) 多 bank 结构和地图

**Independent Test**: 使用 examples/quest.nesl (UNROM 4 banks, 6 worlds)，检查反编译输出包含 `const WORLDS: [map] = [W1..W6]` 和每个地图定义

### Tests for User Story 3 (TDD) ⚠️

- [ ] T071 [P] [US3] Write failing test `unrom_bank_detection` in `crates/nesl/tests/decompile.rs`: load quest.nes ROM → assert mapper==2 → assert prg_banks==4 (from header byte 4)
- [ ] T072 [P] [US3] Write failing test `unrom_map_list_restoration` in `crates/nesl/tests/decompile.rs`: decompile quest.nes → assert output contains "const WORLDS: [map] = [W1, W2, W3, W4, W5, W6];"
- [ ] T073 [P] [US3] Write failing test `roundtrip_quest_unrom` in `crates/nesl/tests/roundtrip.rs`: compile quest.nesl with --debug-symbols → decompile → recompile → run both ROMs for 300 frames with input (RIGHT held) → assert hero x position > 100 in both, assert gold collected count identical

### UNROM Support in ROM Parser (US3)

- [ ] T074 [US3] Add UNROM bank extraction to `load_rom()` in `crates/nesl/src/decompile/rom_parser.rs`: if mapper==2, calculate `prg_banks = rom[4] as usize`, split `prg_rom` into banks of 16KB each, store as `Vec<Vec<u8>>` in RomImage (change prg_rom to `prg_banks: Vec<Vec<u8>>`)
- [ ] T075 [US3] Add fixed bank identification: for UNROM, last bank is fixed at $C000-$FFFF, earlier banks are switchable at $8000-$BFFF, annotate in RomImage

### Symbol Table for Banked Resources (US3)

- [ ] T076 [US3] Update map extraction in `generate_symbols()` for UNROM: when compiling with banked=true, for each map in bank N, set `location: Bank { bank: N, addr: local_addr_in_bank, length }` instead of PrgRom
- [ ] T077 [US3] Add map list constant extraction: if `out` contains `WORLDS` or similar [map] array, create Constant entry with special marker `is_map_list: true`, store map indices in `data` field
- [ ] T078 [US3] Add bank info to Function entries: for functions in switchable banks, set `bank: Some(N)` field (add to Function struct)

### Banked Resource Restoration (US3)

- [ ] T079 [US3] Update map restoration in `build_ast()` for Bank location: read map data from `rom.prg_banks[bank][addr - 0x8000..]`, convert to TopLevel::Map with bank annotation in comment
- [ ] T080 [US3] Implement map list restoration: for Constant with `is_map_list`, parse map indices from `data`, look up map names from `symbols.resources`, create `TopLevel::Const { name: "WORLDS", ty: "[map]", value: ConstValue::MapList(vec!["W1", "W2", ...]) }` (extend ConstValue enum)
- [ ] T081 [US3] Add bank comment to map definitions: in `generate_source()`, for maps with bank>0, prepend comment `// Bank <N>` before map definition

### Validation for US3

- [ ] T082 [US3] Run test `unrom_bank_detection`: assert ROM parsed correctly, prg_banks.len() == 4
- [ ] T083 [US3] Run test `unrom_map_list_restoration`: assert decompiled code has "const WORLDS: [map] = [W1, W2, W3, W4, W5, W6];" and 6 map definitions
- [ ] T084 [US3] Run test `roundtrip_quest_unrom`: assert nes-sim runs both ROMs identically, hero position and game state match after 300 frames
- [ ] T085 [US3] Manual validation: run `nesc build examples/quest.nesl --debug-symbols && nesc decompile examples/quest.nes -o /tmp/quest.dec.nesl`, verify all 6 worlds present, recompile works

**Checkpoint**: User Story 3 完成 — UNROM 支持，大型游戏可反编译

---

## Phase 6: User Story 4 - MMC3 IRQ Handler Decompilation (Priority: P3)

**Goal**: 还原 MMC3 (mapper 4) IRQ 中断处理函数

**Independent Test**: 使用 examples/split.nesl (MMC3 分屏)，检查反编译输出包含 `fn irq_handler() { ... }` 和 MMC3 寄存器操作

### Tests for User Story 4 (TDD) ⚠️

- [ ] T086 [P] [US4] Write failing test `mmc3_irq_detection` in `crates/nesl/tests/decompile.rs`: load split.nes ROM → assert mapper==4 → assert IRQ vector (at rom[prg_size-4..prg_size-2]) != default stub address
- [ ] T087 [P] [US4] Write failing test `mmc3_irq_handler_restoration` in `crates/nesl/tests/decompile.rs`: decompile split.nes → assert output contains "fn irq_handler()" and function body (even if simplified)
- [ ] T088 [P] [US4] Write failing test `roundtrip_split_mmc3` in `crates/nesl/tests/roundtrip.rs`: compile split.nesl with --debug-symbols → decompile → recompile → run both ROMs for 60 frames → assert scanline split triggered (check PPU register writes in nes-sim)

### MMC3 Support in ROM Parser (US4)

- [ ] T089 [US4] Add MMC3 ROM structure to `load_rom()`: if mapper==4, parse PRG banks (2 switchable 8KB + 1 fixed 16KB), parse CHR banks (2KB/1KB configurable), read IRQ vector from last 6 bytes, store in RomImage
- [ ] T090 [US4] Add IRQ vector extraction: read 3 vectors (NMI at -6, RESET at -4, IRQ at -2 from ROM end), add `irq_addr: u16` field to RomImage

### Symbol Table for IRQ Handler (US4)

- [ ] T091 [US4] Add IRQ handler extraction to `generate_symbols()`: if `out.irq_handler` exists (check compiler output), create Function entry with name "irq_handler", addr from IRQ vector, params=[], return_ty=None, mark as special handler
- [ ] T092 [US4] Add MMC3 register mapping to builtins: hardcode MMC3 register addresses ($8000/$8001/$A000/$A001/$C000/$C001/$E000/$E001) as special pseudo-functions "__mmc3_bank_select", "__mmc3_bank_data", etc.

### IRQ Handler Restoration (US4)

- [ ] T093 [US4] Implement IRQ handler restoration in `build_ast()`: if `symbols.functions` contains "irq_handler", disassemble function body (simple pattern: writes to MMC3 registers, RTI), create TopLevel::Function with simplified body (e.g. single comment "// MMC3 IRQ: scanline split" if full disassembly not feasible in v1)
- [ ] T094 [US4] Add placeholder for complex IRQ body: if IRQ handler has >10 instructions, output comment `// Complex IRQ handler at $<addr> - manual review needed` instead of attempting full disassembly

### Code Generation for IRQ Handler (US4)

- [ ] T095 [US4] Add IRQ handler formatting in `generate_source()`: output `fn irq_handler() {\n  // <body or comment>\n}` at end of functions section
- [ ] T096 [US4] Add MMC3-specific comments: if mapper==4, prepend `// MMC3 (Mapper 4) ROM` comment at top of file

### Validation for US4

- [ ] T097 [US4] Run test `mmc3_irq_detection`: assert IRQ vector extracted correctly from ROM
- [ ] T098 [US4] Run test `mmc3_irq_handler_restoration`: assert "fn irq_handler()" present in output
- [ ] T099 [US4] Run test `roundtrip_split_mmc3`: assert scanline split behavior identical in both ROMs (check PPUCTRL writes during vblank)
- [ ] T100 [US4] Manual validation: run `nesc build examples/split.nesl --debug-symbols && nesc decompile examples/split.nes -o /tmp/split.dec.nesl`, verify IRQ handler present, recompile works

**Checkpoint**: User Story 4 完成 — MMC3 支持，所有 3 种 mapper 可反编译

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: 错误处理、性能、文档和最终验收

### Error Handling & Edge Cases

- [ ] T101 [P] Implement non-NESL ROM detection in `decompile()`: if symbols file missing and no magic number in ROM, return `NotNeslRom` error with message "此 ROM 不包含 NESL 调试符号，无法反编译"
- [ ] T102 [P] Implement version check in `parse_symbols()`: if `symbols.version` major != "1", return `UnsupportedVersion { found, supported: "1.x.x" }` error
- [ ] T103 [P] Add mapper validation: if ROM mapper not in [0, 2, 4], return `UnsupportedMapper { mapper }` error
- [ ] T104 [P] Add address range validation in `build_ast()`: for each variable, assert `addr` in RAM range (0x0000-0x07FF), return `SymbolMismatch { reason: "variable <name> address <addr> out of RAM range" }` if invalid
- [ ] T105 Add incomplete symbols handling: if `symbols.variables` missing entries (check against expected count from spec), log warning but continue, output placeholder variables as `// var <placeholder_name>: u8; // address unknown`

### Performance Optimization

- [ ] T106 [P] Optimize symbol table parsing: use `serde_json::from_reader()` instead of reading entire file to String, test with 128KB quest.syms file
- [ ] T107 [P] Add progress indicator to CLI: for large ROMs (>64KB), print "正在解析 ROM...", "正在重建 AST...", "正在生成源码..." to stderr during decompile
- [ ] T108 Benchmark decompile performance: run `time nesc decompile examples/quest.nes -o /tmp/quest.dec.nesl`, assert <5 seconds on typical dev machine, record in quickstart.md

### Documentation & CLI UX

- [ ] T109 [P] Add CLI help text to `crates/nesc/src/cmd/decompile.rs`: usage, examples ("nesc decompile hello.nes", "nesc decompile game.nes --symbols custom.syms -o out.nesl"), flags description
- [ ] T110 [P] Add verbose mode (`-v` flag): print warnings (incomplete symbols, unrecognized patterns) to stderr, normal mode only prints errors
- [ ] T111 [P] Update README.md: add "Decompiler" section with quickstart example, link to contracts/decompile-api.md
- [ ] T112 Add inline code comments: in `generate_source()`, optionally add `// 原地址: $<addr>` comments after variable/function declarations if `--annotate` flag passed (not in MVP)

### Final Validation (from quickstart.md)

- [ ] T113 Run all roundtrip tests: `cargo test --package nesl roundtrip` → assert all 4 user stories' examples pass (hello, sprite/game, quest, split)
- [ ] T114 Run all error handling tests: `cargo test --package nesl decompile::error` → assert NotNeslRom, SymbolsNotFound, VersionMismatch, UnsupportedMapper errors work
- [ ] T115 Run manual validation checklist from quickstart.md: compile each example with --debug-symbols, decompile, verify output readability, recompile, compare ROM behavior
- [ ] T116 Performance validation: decompile quest.nes (128KB UNROM), assert time <5 seconds, memory <100MB
- [ ] T117 Code quality check: run `cargo clippy --package nesl -- -D warnings`, fix all warnings in decompile/ module
- [ ] T118 Run `cargo test --workspace` to ensure no regressions in existing compiler tests

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖，立即开始
- **Foundational (Phase 2)**: 依赖 Setup 完成 — **阻塞所有用户故事**
- **User Stories (Phase 3-6)**: 所有依赖 Foundational 完成
  - US1 (P1): 可在 Foundational 后立即开始 — 无其他故事依赖
  - US2 (P2): 可在 Foundational 后立即开始 — 可能集成 US1 但独立可测
  - US3 (P3): 可在 Foundational 后立即开始 — 扩展 US1/US2 但独立可测
  - US4 (P3): 可在 Foundational 后立即开始 — 独立的 mapper 支持
- **Polish (Phase 7)**: 依赖所有期望的用户故事完成

### User Story Dependencies

- **User Story 1 (P1)**: 基础往返 — 无依赖
- **User Story 2 (P2)**: 资源还原 — 依赖 US1 的符号表和 AST 框架，但可独立测试
- **User Story 3 (P3)**: UNROM 多 bank — 扩展 US2 的地图还原，但可独立测试
- **User Story 4 (P3)**: MMC3 IRQ — 独立的 mapper 支持，不依赖其他故事

### Within Each User Story

- Tests → 模型/数据结构 → 服务/逻辑 → 集成
- TDD 强制：测试必须先写且失败，实现后才能通过

### Parallel Opportunities

- Phase 1 所有任务标记 [P] 可并行
- Phase 2 数据结构定义 (T009-T020) 可并行
- 一旦 Foundational 完成，US1/US2/US3/US4 可并行开发（如有团队容量）
- 每个故事内的测试 (标记 [P]) 可并行编写
- Phase 7 的 [P] 任务（错误处理、性能、文档）可并行

---

## Parallel Example: User Story 2

```bash
# 并行启动 US2 的所有测试：
Task: "Write failing test tile_restoration in crates/nesl/tests/decompile.rs"
Task: "Write failing test map_restoration in crates/nesl/tests/decompile.rs"
Task: "Write failing test roundtrip_sprite_example in crates/nesl/tests/roundtrip.rs"

# 并行实现符号表提取：
Task: "Add tile extraction to generate_symbols()"
Task: "Add map extraction to generate_symbols()"
Task: "Add palette extraction to generate_symbols()"

# 并行实现代码生成：
Task: "Implement tile formatting in generate_source()"
Task: "Implement map formatting in generate_source()"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. 完成 Phase 1: Setup
2. 完成 Phase 2: Foundational (关键 — 阻塞所有故事)
3. 完成 Phase 3: User Story 1 (基本往返)
4. **STOP and VALIDATE**: 测试 US1 独立运行，hello.nesl 往返成功
5. 可选：部署/演示 MVP

### Incremental Delivery

1. Setup + Foundational → 基础就绪
2. 添加 US1 → 独立测试 → 部署/演示（MVP！）
3. 添加 US2 → 独立测试 → 部署/演示（资源还原）
4. 添加 US3 → 独立测试 → 部署/演示（UNROM 支持）
5. 添加 US4 → 独立测试 → 部署/演示（MMC3 支持）
6. 每个故事增加价值，不破坏之前的故事

### Parallel Team Strategy

多人开发：

1. 团队一起完成 Setup + Foundational
2. Foundational 完成后：
   - Developer A: User Story 1 (基础往返)
   - Developer B: User Story 2 (资源还原)
   - Developer C: User Story 3 (UNROM) 或 User Story 4 (MMC3)
3. 故事独立完成和集成

---

## Notes

- [P] 任务 = 不同文件，无依赖
- [Story] 标签将任务映射到具体用户故事，便于追踪
- 每个用户故事应独立完成和测试
- TDD 强制：先验证测试失败再实现
- 每个任务或逻辑组后提交
- 在任何 checkpoint 停下验证故事独立性
- 避免：模糊任务、同文件冲突、破坏独立性的跨故事依赖

---

## Task Count Summary

- **Phase 1 (Setup)**: 8 tasks
- **Phase 2 (Foundational)**: 21 tasks
- **Phase 3 (US1 - Round-trip)**: 25 tasks
- **Phase 4 (US2 - Resources)**: 16 tasks
- **Phase 5 (US3 - UNROM)**: 15 tasks
- **Phase 6 (US4 - MMC3)**: 15 tasks
- **Phase 7 (Polish)**: 18 tasks

**Total**: 118 tasks

**MVP Scope** (推荐): Phase 1 + Phase 2 + Phase 3 (54 tasks) — 基本往返功能，符号表生成，核心反编译

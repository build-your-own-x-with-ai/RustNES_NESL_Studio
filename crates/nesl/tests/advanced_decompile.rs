//! Test advanced decompiler variable analysis

use nesl::decompile::advanced::{
    scan_zeropage_accesses, infer_variables,
    scan_function_calls, identify_interrupt_handlers, infer_functions
};

#[test]
fn test_variable_scanning() {
    // Load a real ROM for testing
    let rom_path = "../../examples/sprite.nes";
    let rom_bytes = std::fs::read(rom_path).expect("Failed to read sprite.nes");

    // Parse ROM to get PRG
    assert_eq!(&rom_bytes[0..4], b"NES\x1a");
    let prg_banks = rom_bytes[4] as usize;
    let prg_size = prg_banks * 16384;
    let prg_rom = &rom_bytes[16..16 + prg_size];

    // Scan Zero Page accesses
    let accesses = scan_zeropage_accesses(prg_rom);

    println!("=== Zero Page Access Scan ===");
    println!("Found {} unique ZP addresses accessed", accesses.len());

    // Should find at least some ZP accesses
    assert!(accesses.len() > 0, "Should find ZP accesses");

    // Show top 10 most accessed
    let mut addr_counts: Vec<_> = accesses.iter()
        .map(|(addr, list)| (*addr, list.len()))
        .collect();
    addr_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    println!("\nTop 10 most accessed ZP addresses:");
    for (addr, count) in addr_counts.iter().take(10) {
        println!("  ${:02X}: {} accesses", addr, count);
    }

    // Infer variables
    let variables = infer_variables(&accesses);

    println!("\n=== Inferred Variables ===");
    println!("Found {} variables", variables.len());

    // Should infer some variables
    assert!(variables.len() > 0, "Should infer variables");

    // Show all variables
    let mut vars: Vec<_> = variables.values().collect();
    vars.sort_by_key(|v| v.addr);

    println!("\nInferred variables:");
    for var in vars {
        println!("  ${:02X} {:12} {:?} - reads:{} writes:{} pattern:{:?}",
            var.addr,
            var.name,
            var.var_type,
            var.read_count,
            var.write_count,
            var.access_pattern
        );
    }
}

#[test]
fn test_variable_types() {
    // Test with sprite.nes which we know has u8 variables
    let rom_bytes = std::fs::read("../../examples/sprite.nes").expect("sprite.nes not found");
    let prg_rom = &rom_bytes[16..16 + 32768];

    let accesses = scan_zeropage_accesses(prg_rom);
    let variables = infer_variables(&accesses);

    // Check that we identified some u8 variables
    let u8_vars: Vec<_> = variables.values()
        .filter(|v| v.var_type == nesl::decompile::advanced::VarType::U8)
        .collect();

    assert!(u8_vars.len() > 0, "Should find u8 variables");

    println!("Found {} u8 variables", u8_vars.len());
}

#[test]
fn test_commercial_rom() {
    // Test with a commercial ROM if available
    let rom_path = "../../roms/FC_ROMS-main/ROM/0942/Solomon's Key (U) [!].nes";

    if let Ok(rom_bytes) = std::fs::read(rom_path) {
        let prg_rom = &rom_bytes[16..16 + 32768];

        let accesses = scan_zeropage_accesses(prg_rom);
        let variables = infer_variables(&accesses);

        println!("=== Solomon's Key Variable Analysis ===");
        println!("ZP addresses accessed: {}", accesses.len());
        println!("Variables inferred: {}", variables.len());

        // Show some interesting variables
        for var in variables.values().filter(|v| v.access_count > 20) {
            println!("  {} (${:02X}): {:?}, {} accesses",
                var.name, var.addr, var.access_pattern, var.access_count);
        }

        assert!(variables.len() > 5, "Should find multiple variables in commercial ROM");
    } else {
        println!("Skipping commercial ROM test (file not found)");
    }
}

#[test]
fn test_function_identification() {
    let rom_bytes = std::fs::read("../../examples/sprite.nes").expect("sprite.nes not found");
    let prg_rom = &rom_bytes[16..16 + 32768];

    println!("=== Function Identification ===");

    // Test interrupt handlers
    let handlers = identify_interrupt_handlers(prg_rom);
    println!("\nInterrupt handlers:");
    for (name, addr) in &handlers {
        println!("  {}: ${:04X}", name, addr);
    }
    assert!(handlers.contains_key("reset"), "Should find RESET vector");

    // Test function calls
    let calls = scan_function_calls(prg_rom);
    println!("\nFunction call graph:");
    println!("  {} unique functions called", calls.len());

    let mut call_counts: Vec<_> = calls.iter()
        .map(|(callee, callers)| (*callee, callers.len()))
        .collect();
    call_counts.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    println!("\nTop 10 most called functions:");
    for (addr, count) in call_counts.iter().take(10) {
        println!("  ${:04X}: called {} times", addr, count);
    }

    // Test full function inference
    let functions = infer_functions(prg_rom);
    println!("\n=== Inferred Functions ===");
    println!("Total functions: {}", functions.len());

    // Show first 15 functions
    println!("\nFunction summary:");
    for (addr, func) in functions.iter().take(15) {
        println!("  ${:04X} {:12} - size:{:4}, calls:{}, called_from:{}, exits:{}",
            addr,
            func.name,
            func.size_bytes,
            func.calls_to.len(),
            func.called_from.len(),
            func.exit_points.len()
        );
    }

    assert!(functions.len() > 5, "Should identify multiple functions");
}

#[test]
fn test_commercial_rom_functions() {
    let rom_path = "../../roms/FC_ROMS-main/ROM/0942/Solomon's Key (U) [!].nes";

    if let Ok(rom_bytes) = std::fs::read(rom_path) {
        let prg_rom = &rom_bytes[16..16 + 32768];

        println!("=== Solomon's Key Function Analysis ===");

        let functions = infer_functions(prg_rom);
        println!("Total functions identified: {}", functions.len());

        // Show interrupt handlers
        println!("\nInterrupt handlers:");
        for func in functions.values().filter(|f| f.name == "reset" || f.name == "nmi" || f.name == "irq") {
            println!("  {} @ ${:04X}", func.name, func.entry);
        }

        // Show most complex functions (most calls)
        let mut funcs: Vec<_> = functions.values().collect();
        funcs.sort_by_key(|f| std::cmp::Reverse(f.calls_to.len()));

        println!("\nMost complex functions (by calls made):");
        for func in funcs.iter().take(10) {
            println!("  ${:04X} {} - makes {} calls, size {} bytes",
                func.entry, func.name, func.calls_to.len(), func.size_bytes);
        }

        // Show most called functions
        funcs.sort_by_key(|f| std::cmp::Reverse(f.called_from.len()));
        println!("\nMost called functions:");
        for func in funcs.iter().take(10) {
            println!("  ${:04X} {} - called {} times",
                func.entry, func.name, func.called_from.len());
        }

        assert!(functions.len() > 20, "Commercial ROM should have many functions");
    } else {
        println!("Skipping commercial ROM test (file not found)");
    }
}

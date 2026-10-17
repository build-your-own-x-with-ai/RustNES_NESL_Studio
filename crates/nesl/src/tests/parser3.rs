//! 003: contextual `map` / `metasprite` declarations.
use crate::ast::*;
use crate::parser::parse;

#[test]
fn map_declaration() {
    let p = parse("map LEVEL = [\"#..#\", \"####\",];").unwrap();
    match &p.items[0] {
        Decl::Map { name, rows, .. } => {
            assert_eq!(name, "LEVEL");
            assert_eq!(rows.iter().map(|(r, _)| r.as_str()).collect::<Vec<_>>(), vec!["#..#", "####"]);
            assert_eq!(rows[1].1.line, 1);
        }
        d => panic!("{d:?}"),
    }
}

#[test]
fn metasprite_declaration() {
    let p = parse("metasprite HERO = [(0, 0, 1), (-8, 8, 2, 0x40)];").unwrap();
    match &p.items[0] {
        Decl::Metasprite { name, parts, .. } => {
            assert_eq!(name, "HERO");
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[1].dx.to_string(), "(neg 8)");
            assert_eq!(parts[1].attr.as_ref().unwrap().to_string(), "64");
            assert!(parts[0].attr.is_none());
        }
        d => panic!("{d:?}"),
    }
}

#[test]
fn map_and_metasprite_stay_usable_as_identifiers() {
    let p = parse("var map: u8; fn metasprite_count() -> u8 { return map; } fn main() { map = 1; }").unwrap();
    assert_eq!(p.items.len(), 3);
    assert!(matches!(&p.items[0], Decl::Var { name, .. } if name == "map"));
}

#[test]
fn malformed_declarations() {
    let e = parse("metasprite M = [(1, 2)];").unwrap_err();
    assert!(e.message.contains("(dx, dy, tile"), "{}", e.message);
    let e = parse("map M = [1, 2];").unwrap_err();
    assert!(e.message.contains("map row string"), "{}", e.message);
}

#[test]
fn palette_zone_declaration() {
    let p = parse("palette_zone W = [\"0123\", \"..33\"]; var palette_zone: u8;").unwrap();
    match &p.items[0] {
        Decl::PaletteZone { name, rows, .. } => {
            assert_eq!(name, "W");
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[1].0, "..33");
        }
        d => panic!("{d:?}"),
    }
    assert!(matches!(&p.items[1], Decl::Var { name, .. } if name == "palette_zone"));
}

#[test]
fn music_declarations() {
    let p = parse("music A = \"C D E\"; music B = [\"O2 C\", \"G\"];").unwrap();
    match &p.items[0] {
        Decl::Music { name, parts, .. } => {
            assert_eq!(name, "A");
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].0, "C D E");
            assert_eq!(parts[0].1.col, 11, "position of the string literal");
        }
        d => panic!("{d:?}"),
    }
    assert!(matches!(&p.items[1], Decl::Music { parts, .. } if parts.len() == 2 && parts[1].0 == "G"));
}

#[test]
fn sound_declarations() {
    let p = parse("sound L = (pulse, 0x30, -4, 10); sound N = (noise, 5, 0, 20, 15);").unwrap();
    match &p.items[0] {
        Decl::Sound { name, channel, args, .. } => {
            assert_eq!(name, "L");
            assert_eq!(channel.0, "pulse");
            assert_eq!(args.iter().map(|a| a.to_string()).collect::<Vec<_>>(), vec!["48", "(neg 4)", "10"]);
        }
        d => panic!("{d:?}"),
    }
    assert!(matches!(&p.items[1], Decl::Sound { args, .. } if args.len() == 4));
}

#[test]
fn music_and_sound_stay_usable_as_identifiers() {
    let p = parse("var music: u8; var sound: u8; fn main() { music = sound; }").unwrap();
    assert_eq!(p.items.len(), 3);
}

#[test]
fn malformed_music_and_sound() {
    assert!(parse("music M = 5;").unwrap_err().message.contains("music string"));
    assert!(parse("sound S = (pulse, 1);").unwrap_err().message.contains("(channel, period, delta, frames"));
    assert!(parse("sound S = (5, 1, 2, 3);").unwrap_err().message.contains("channel"));
}

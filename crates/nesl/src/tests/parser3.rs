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

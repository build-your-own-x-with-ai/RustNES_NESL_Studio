use crate::lexer::{lex, Tok};

fn kinds(src: &str) -> Vec<Tok> {
    lex(src).expect("lex").into_iter().map(|t| t.tok).collect()
}

#[test]
fn numbers_in_all_bases_chars_and_bools() {
    assert_eq!(
        kinds("12 0xFF 0b1010 1_000 'A' true false"),
        vec![
            Tok::Int(12), Tok::Int(255), Tok::Int(10), Tok::Int(1000), Tok::Int(65),
            Tok::Int(1), Tok::Int(0), Tok::Eof
        ]
    );
}

#[test]
fn keywords_identifiers_and_longest_match_operators() {
    assert_eq!(
        kinds("fn main_1 <<= << <= < -> .. && ||"),
        vec![
            Tok::Kw("fn"), Tok::Ident("main_1".into()), Tok::Sym("<<="), Tok::Sym("<<"),
            Tok::Sym("<="), Tok::Sym("<"), Tok::Sym("->"), Tok::Sym(".."), Tok::Sym("&&"),
            Tok::Sym("||"), Tok::Eof
        ]
    );
}

#[test]
fn strings_with_escapes() {
    assert_eq!(kinds(r#""HI\n\"\x41\0""#), vec![Tok::Str(vec![b'H', b'I', 10, b'"', 0x41, 0]), Tok::Eof]);
}

#[test]
fn comments_are_skipped_and_positions_tracked() {
    let toks = lex("// c\n  /* a\n b */ x").unwrap();
    assert_eq!(toks[0].tok, Tok::Ident("x".into()));
    assert_eq!((toks[0].pos.line, toks[0].pos.col), (3, 7));
}

#[test]
fn errors_have_positions() {
    let e = lex("let a = \"open").unwrap_err();
    assert_eq!((e.line, e.col), (1, 9));
    assert!(e.message.contains("unterminated"));
    let e = lex("x = 1 $").unwrap_err();
    assert_eq!((e.line, e.col), (1, 7));
    let e = lex("0x1FFFFFFFF").unwrap_err();
    assert!(e.message.contains("too large"));
}

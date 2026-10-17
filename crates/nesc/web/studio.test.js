// Unit tests for studio.js (run with `node --test`; cargo test runs them via tests/studio.rs).
"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const S = require(require("node:path").join(__dirname, "studio.js"));

const SRC = `// tile 9 = ["no"]; only a comment
const PAL: [u8] = [0x0F, 0x30, 16, LIGHT + 1];
tile '#' = [
    "33333333",
    "3......3",
    "3.1111.3",
    "3.1221.3",
    "3.1221.3",
    "3.1111.3",
    "3......3",
    "33333333",
];
tile 2 = ["........", "...33...", "..3..3..", "........", "........", "........", "........", "........"];
map W = [
    "#..]..#",   // a ']' inside a row
    // a comment between rows
    "#.....#",
];
fn main() { load_palette(PAL); draw_map(W); }
`;

const BUILTINS = {
  functions: [
    { name: "sprite", sig: "sprite(i, x, y, tile, attr)", doc: "..." },
    { name: "sfx", sig: "sfx(id: u8)", doc: "..." },
    { name: "print", sig: "print(x: u8, y: u8, text)", doc: "..." },
    { name: "print_u8", sig: "print_u8(x: u8, y: u8, v: u8)", doc: "..." },
    { name: "stop_sound", sig: "stop_sound()", doc: "..." },
  ],
  constants: ["BTN_A", "BTN_B", "SFX_JUMP"],
  keywords: ["loop", "let", "fn"],
};

const grid = (rows) => rows.map((r) => [...r].map((c) => "0123".indexOf(c === "." ? "0" : c)));

test("findTiles skips comments and parses index forms and pixels", () => {
  const t = S.findTiles(SRC);
  assert.equal(t.length, 2);
  assert.equal(t[0].indexText, "'#'");
  assert.equal(t[0].index, 35);
  assert.equal(t[0].pixels[1][1], 0);
  assert.equal(t[0].pixels[2][2], 1);
  assert.equal(t[1].index, 2);
  assert.equal(t[1].pixels[1][3], 3);
});

test("upsertTile changes only the edited row", () => {
  const t = S.findTiles(SRC)[0];
  const px = t.pixels.map((r) => r.slice());
  px[3][3] = 3;
  const out = S.upsertTile(SRC, "'#'", px);
  const a = SRC.split("\n"), b = out.split("\n");
  assert.equal(a.length, b.length);
  const changed = a.map((l, i) => l !== b[i]).filter(Boolean).length;
  assert.equal(changed, 1);
  assert.ok(out.includes('"3.1321.3"'));
});

test("upsertTile matches indexes by value and appends new tiles", () => {
  const two = S.upsertTile(SRC, "0x02", grid(Array(8).fill("11111111")));
  assert.equal(S.findTiles(two).length, 2);
  assert.equal(S.findTiles(two)[1].pixels[0][0], 1);
  const seven = S.upsertTile(SRC, "7", grid(["3.......", ...Array(7).fill("........")]));
  assert.ok(seven.startsWith(SRC), "appended, nothing else touched");
  const t = S.findTiles(seven);
  assert.equal(t.length, 3);
  assert.equal(t[2].index, 7);
  assert.equal(t[2].rows.length, 8);
  assert.equal(t[2].pixels[0][0], 3);
});

test("findMaps handles brackets in rows and comments between rows", () => {
  const m = S.findMaps(SRC);
  assert.equal(m.length, 1);
  assert.equal(m[0].name, "W");
  assert.deepEqual(m[0].rows.map((r) => r.text), ["#..]..#", "#.....#"]);
});

test("setMapCell edits, pads and adds rows", () => {
  const rows = (src) => S.findMaps(src)[0].rows.map((r) => r.text);
  assert.equal(rows(S.setMapCell(SRC, "W", 2, 1, "$"))[1], "#.$...#");
  assert.equal(rows(S.setMapCell(SRC, "W", 10, 0, "@"))[0], "#..]..#...@");
  assert.deepEqual(rows(S.setMapCell(SRC, "W", 0, 3, "#")), ["#..]..#", "#.....#", "", "#"]);
  assert.throws(() => S.setMapCell(SRC, "W", 0, 0, '"'));
  assert.throws(() => S.setMapCell(SRC, "W", 0, 30, "#"));
  assert.throws(() => S.setMapCell(SRC, "Q", 0, 0, "#"));
});

test("findPalette follows load_palette and setPaletteEntry writes hex", () => {
  const p = S.findPalette(SRC);
  assert.equal(p.name, "PAL");
  assert.deepEqual(p.entries.map((e) => e.value), [0x0f, 0x30, 16, null]);
  assert.ok(S.setPaletteEntry(SRC, 1, 0x2a).includes("[0x0F, 0x2A, 16, LIGHT + 1]"));
  assert.throws(() => S.setPaletteEntry(SRC, 3, 0x21), /expression/);
  const longer = S.setPaletteEntry(SRC, 5, 0x21);
  assert.deepEqual(S.findPalette(longer).entries.map((e) => e.value), [0x0f, 0x30, 16, null, 0x0f, 0x21]);
  assert.throws(() => S.setPaletteEntry(SRC, 0, 0x40));
  assert.equal(S.findPalette("fn main() {}"), null);
});

test("wordAt and prefixAt", () => {
  assert.equal(S.wordAt("let pad = joypad(0);", 12).word, "joypad");
  assert.equal(S.wordAt("let pad = joypad(0);", 16).word, "joypad", "cursor right after the word");
  assert.equal(S.wordAt("a + b", 2), null);
  assert.equal(S.prefixAt("    spr", 7), "spr");
  assert.equal(S.prefixAt("x + 1", 5), "");
});

test("complete ranks built-ins, user names, constants and keywords", () => {
  const first = S.complete("spr", BUILTINS, SRC)[0];
  assert.equal(first.label, "sprite");
  assert.equal(first.detail, "sprite(i, x, y, tile, attr)");
  assert.equal(first.call, true);
  assert.deepEqual(S.complete("BTN", BUILTINS, SRC).map((c) => c.label), ["BTN_A", "BTN_B"]);
  assert.deepEqual(S.complete("pr", BUILTINS, SRC).map((c) => c.label).slice(0, 2), ["print", "print_u8"]);
  assert.ok(S.complete("lo", BUILTINS, SRC).some((c) => c.label === "loop" && c.kind === "keyword"));
  const user = S.complete("P", BUILTINS, SRC).map((c) => c.label);
  assert.ok(user.includes("PAL"), user.join());
  assert.ok(S.complete("ma", BUILTINS, SRC).some((c) => c.label === "main"));
  assert.equal(S.complete("stop", BUILTINS, SRC)[0].args, false, "no-argument call");
});

test("decodeChr reads 512 tiles of 2-bit pixels", () => {
  const rom = new Uint8Array(16 + 32768 + 8192);
  rom.set([0x4e, 0x45, 0x53, 0x1a, 2, 1]);
  const chr = 16 + 32768;
  rom[chr + 16] = 0x80; // tile 1 row 0 plane 0
  rom[chr + 24] = 0x80; // tile 1 row 0 plane 1
  rom[chr + 25] = 0x01; // tile 1 row 1 plane 1, rightmost pixel
  const tiles = S.decodeChr(rom);
  assert.equal(tiles.length, 512);
  assert.equal(tiles[1][0], 3);
  assert.equal(tiles[1][15], 2);
  assert.equal(tiles[1][1], 0);
});

test("NES_RGB is the jsnes colour table", () => {
  assert.equal(S.NES_RGB.length, 64);
  assert.deepEqual(S.NES_RGB[0x30], [255, 255, 255]);
  assert.deepEqual(S.NES_RGB[0x0f], [0, 0, 0]);
  assert.deepEqual(S.NES_RGB[0x16], [0xbc, 0x19, 0x00]);
});

// Runs an iNES ROM in jsnes (headless) and writes its last frame for cross-checking.
//
//   node run.js <rom.nes> <frames> <buttons> <out.txt>
//
// buttons: controller 1 mask held from power-on (NESL bit order: $80 = A ... $01 = Right).
// out.txt: line 1 = jsnes' 64-colour table, line 2 = 256*240 pixels (same integer format).
// Pixels jsnes left undefined are written as 4294967295 (see xcheck.rs).
"use strict";
const fs = require("fs");
const jsnes = require("jsnes");

const [romPath, framesArg, buttonsArg, outPath] = process.argv.slice(2);
if (!outPath) {
  console.error("usage: node run.js <rom.nes> <frames> <buttons> <out.txt>");
  process.exit(2);
}
let last = null;
const nes = new jsnes.NES({ emulateSound: false, onFrame: (buf) => { last = buf; } });
nes.ppu.clipToTvSize = false; // compare the whole 256x240 picture
nes.loadROM(fs.readFileSync(romPath).toString("latin1"));
const buttons = parseInt(buttonsArg, 10) || 0;
for (let i = 0; i < 8; i++) {
  // jsnes button i: A, B, SELECT, START, UP, DOWN, LEFT, RIGHT
  if (buttons & (0x80 >> i)) nes.buttonDown(1, i);
}
const frames = parseInt(framesArg, 10);
for (let f = 0; f < frames; f++) nes.frame();
const table = nes.ppu.palTable.curTable;
fs.writeFileSync(outPath, table.join(" ") + "\n" + Array.from(last, (v) => (v === undefined ? 4294967295 : v)).join(" ") + "\n");

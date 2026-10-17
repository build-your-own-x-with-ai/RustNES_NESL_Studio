// Test helper: applies a studio.js edit to a source file.
//   node studio_edit.js <studio.js> <in.nesl> <out.nesl> <expression using S and src>
"use strict";
const fs = require("fs");
const [studio, inp, out, expr] = process.argv.slice(2);
const S = require(studio);
const src = fs.readFileSync(inp, "utf8");
fs.writeFileSync(out, new Function("S", "src", "return " + expr)(S, src));

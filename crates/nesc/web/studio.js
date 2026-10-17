// NESL Studio editing logic: pure functions shared by the page (window.Studio) and the
// Node tests (require). Every edit returns new source text and only rewrites the text
// range of the declaration being edited; other code, comments and layout stay as they are.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory();
  else root.Studio = factory();
})(typeof self !== "undefined" ? self : this, function () {
  "use strict";

  // jsnes 1.2.1 colour table (0xBBGGRR), so previews match the in-page emulator.
  const JSNES_TABLE = [
    0x525252, 0xb40000, 0xa00000, 0xb1003d, 0x740069, 0x00005b, 0x00005f, 0x001840, 0x002f10, 0x084a08, 0x006700,
    0x124200, 0x6d2800, 0x000000, 0x000000, 0x000000, 0xc4d5e7, 0xff4000, 0xdc0e22, 0xff476b, 0xd7009f, 0x680ad7,
    0x0019bc, 0x0054b1, 0x006a5b, 0x008c03, 0x00ab00, 0x2c8800, 0xa47200, 0x000000, 0x000000, 0x000000, 0xf8f8f8,
    0xffab3c, 0xff7981, 0xff5bc5, 0xff48f2, 0xdf49ff, 0x476dff, 0x00b4f7, 0x00e0ff, 0x00e375, 0x03f42b, 0x78b82e,
    0xe5e218, 0x787878, 0x000000, 0x000000, 0xffffff, 0xfff2be, 0xf8b8b8, 0xf8b8d8, 0xffb6ff, 0xffc3ff, 0xc7d1ff,
    0x9adaff, 0x88edf8, 0x83ffdd, 0xb8f8b8, 0xf5f8ac, 0xffffb0, 0xf8d8f8, 0x000000, 0x000000,
  ];
  const NES_RGB = JSNES_TABLE.map((c) => [c & 0xff, (c >> 8) & 0xff, (c >> 16) & 0xff]);

  const isWord = (c) => c !== undefined && /[A-Za-z0-9_]/.test(c);

  /** Source with comments blanked out (same length; newlines and string literals kept). */
  function mask(src) {
    const out = src.split("");
    let i = 0;
    while (i < src.length) {
      const c = src[i], d = src[i + 1];
      if (c === "/" && d === "/") {
        while (i < src.length && src[i] !== "\n") out[i++] = " ";
      } else if (c === "/" && d === "*") {
        while (i < src.length && !(src[i] === "*" && src[i + 1] === "/")) {
          if (src[i] !== "\n") out[i] = " ";
          i++;
        }
        if (i < src.length) {
          out[i] = out[i + 1] = " ";
          i += 2;
        }
      } else if (c === '"' || c === "'") {
        i = skipLiteral(src, i);
      } else {
        i++;
      }
    }
    return out.join("");
  }

  /** Index just past the string / char literal starting at `i`. */
  function skipLiteral(src, i) {
    const q = src[i];
    i++;
    while (i < src.length && src[i] !== q && src[i] !== "\n") i += src[i] === "\\" ? 2 : 1;
    return i + 1;
  }

  /** String literals inside `[...]` opening at `open`: content ranges and the closing bracket. */
  function scanStrings(m, open) {
    const strings = [];
    let i = open + 1;
    while (i < m.length) {
      if (m[i] === '"') {
        const end = skipLiteral(m, i) - 1;
        strings.push({ start: i + 1, end });
        i = end + 1;
      } else if (m[i] === "'") {
        i = skipLiteral(m, i);
      } else if (m[i] === "]") {
        return { strings, close: i };
      } else {
        i++;
      }
    }
    return null;
  }

  /** Numeric or character literal value, or null. */
  function literal(text) {
    const t = text.trim();
    if (/^'(.|\\.)'$/.test(t)) {
      const ch = t.length === 3 ? t[1] : { n: "\n", t: "\t", "0": "\0", "\\": "\\", "'": "'" }[t[2]];
      return ch === undefined ? null : ch.charCodeAt(0);
    }
    const s = t.replace(/_/g, "");
    if (/^0x[0-9a-f]+$/i.test(s)) return parseInt(s.slice(2), 16);
    if (/^0b[01]+$/i.test(s)) return parseInt(s.slice(2), 2);
    if (/^[0-9]+$/.test(s)) return parseInt(s, 10);
    return null;
  }

  const pixel = (c) => (c === "1" ? 1 : c === "2" ? 2 : c === "3" ? 3 : 0);
  const rowText = (row) => row.map((v) => (v === 0 ? "." : String(v))).join("");

  function replaceAll(src, edits) {
    // edits: [{start, end, text}], applied back to front so offsets stay valid
    edits.sort((a, b) => b.start - a.start);
    for (const e of edits) src = src.slice(0, e.start) + e.text + src.slice(e.end);
    return src;
  }

  function declEnd(src, close) {
    let i = close + 1;
    while (i < src.length && (src[i] === " " || src[i] === "\t")) i++;
    return src[i] === ";" ? i + 1 : close + 1;
  }

  // ------------------------------------------------------------------ tiles

  function findTiles(src) {
    const m = mask(src);
    const re = /\btile\s+('(?:\\.|[^'\\])'|[A-Za-z0-9_]+)\s*=\s*\[/g;
    const out = [];
    let hit;
    while ((hit = re.exec(m))) {
      const list = scanStrings(m, re.lastIndex - 1);
      if (!list) continue;
      const rows = list.strings.map((s) => ({ text: src.slice(s.start, s.end), start: s.start, end: s.end }));
      out.push({
        indexText: hit[1],
        index: literal(hit[1]),
        rows,
        pixels: rows.map((r) => [...r.text].map(pixel)),
        start: hit.index,
        end: declEnd(src, list.close),
      });
      re.lastIndex = list.close + 1;
    }
    return out;
  }

  function tileToCode(indexText, pixels) {
    return `tile ${indexText} = [\n${pixels.map((r) => `    "${rowText(r)}",`).join("\n")}\n];`;
  }

  function checkPixels(pixels) {
    const ok = pixels.length === 8 && pixels.every((r) => r.length === 8 && r.every((v) => v >= 0 && v <= 3 && v === (v | 0)));
    if (!ok) throw new Error("a tile is 8x8 pixels with colours 0-3");
  }

  /** Writes `pixels` into the tile with that index (by value), or appends a new tile. */
  function upsertTile(src, indexText, pixels) {
    checkPixels(pixels);
    const want = literal(indexText);
    const t = findTiles(src).find((x) => (want !== null ? x.index === want : x.indexText === indexText.trim()));
    if (!t) {
      const sep = src === "" || src.endsWith("\n") ? "" : "\n";
      return src + sep + tileToCode(indexText.trim(), pixels) + "\n";
    }
    if (t.rows.length !== 8 || t.rows.some((r) => r.text.length !== 8)) {
      return replaceAll(src, [{ start: t.start, end: t.end, text: tileToCode(t.indexText, pixels) }]);
    }
    const edits = [];
    t.rows.forEach((r, i) => {
      if (t.pixels[i].some((v, j) => v !== pixels[i][j])) edits.push({ start: r.start, end: r.end, text: rowText(pixels[i]) });
    });
    return replaceAll(src, edits);
  }

  // ------------------------------------------------------------------ maps

  function findMaps(src) {
    const m = mask(src);
    const re = /\bmap\s+([A-Za-z_]\w*)\s*=\s*\[/g;
    const out = [];
    let hit;
    while ((hit = re.exec(m))) {
      const open = re.lastIndex - 1;
      const list = scanStrings(m, open);
      if (!list) continue;
      out.push({
        name: hit[1],
        rows: list.strings.map((s) => ({ text: src.slice(s.start, s.end), start: s.start, end: s.end })),
        open,
        close: list.close,
        start: hit.index,
        end: declEnd(src, list.close),
      });
      re.lastIndex = list.close + 1;
    }
    return out;
  }

  /** Sets map `name` cell (x, y) to the single character `ch`, padding with `.` as needed. */
  function setMapCell(src, name, x, y, ch) {
    if (typeof ch !== "string" || ch.length !== 1 || ch < " " || ch > "~" || ch === '"' || ch === "\\") {
      throw new Error("a map cell is one printable ASCII character (not \" or \\)");
    }
    if (!(x >= 0 && x <= 254 && y >= 0 && y <= 29)) throw new Error("map cells are 0-254 x 0-29");
    const map = findMaps(src).find((mm) => mm.name === name);
    if (!map) throw new Error(`no map named ${name}`);
    const cell = (text) => {
      const padded = text.padEnd(x + 1, ".");
      return padded.slice(0, x) + ch + padded.slice(x + 1);
    };
    if (y < map.rows.length) {
      const r = map.rows[y];
      return replaceAll(src, [{ start: r.start, end: r.end, text: cell(r.text) }]);
    }
    const added = [];
    for (let i = map.rows.length; i <= y; i++) added.push(i === y ? cell("") : "");
    if (map.rows.length === 0) {
      return replaceAll(src, [{ start: map.open + 1, end: map.open + 1, text: "\n" + added.map((r) => `    "${r}",\n`).join("") }]);
    }
    const last = map.rows[map.rows.length - 1];
    const lineStart = src.lastIndexOf("\n", last.start) + 1;
    const indent = (src.slice(lineStart).match(/^\s*/) || [""])[0];
    const text = added.map((r) => `,\n${indent}"${r}"`).join("");
    return replaceAll(src, [{ start: last.end + 1, end: last.end + 1, text }]);
  }

  // ------------------------------------------------------------------ palette

  /** The constant array passed to the first `load_palette(...)`, with its entries. */
  function findPalette(src) {
    const m = mask(src);
    const call = /\bload_palette\s*\(\s*([A-Za-z_]\w*)/.exec(m);
    if (!call) return null;
    const name = call[1];
    const decl = new RegExp(`\\bconst\\s+${name}\\s*:\\s*\\[\\s*u8\\s*\\]\\s*=\\s*\\[`).exec(m);
    if (!decl) return null;
    const open = decl.index + decl[0].length - 1;
    const entries = [];
    let depth = 0, seg = open + 1, i = open + 1;
    const push = (end) => {
      const raw = src.slice(seg, end);
      const lead = raw.length - raw.trimStart().length;
      const text = raw.trim();
      if (text) entries.push({ text, start: seg + lead, end: seg + lead + text.length, value: literal(text) });
    };
    while (i < m.length) {
      const c = m[i];
      if (c === '"' || c === "'") {
        i = skipLiteral(m, i);
        continue;
      }
      if (c === "(" || c === "[") depth++;
      else if (c === ")") depth--;
      else if (c === "]" && depth > 0) depth--;
      else if (c === "]" || (c === "," && depth === 0)) {
        push(i);
        if (c === "]") return { name, entries, open, close: i };
        seg = i + 1;
      }
      i++;
    }
    return null;
  }

  const hex = (v) => "0x" + v.toString(16).toUpperCase().padStart(2, "0");

  /** Sets palette entry `i` (0-31) to NES colour `value` (0x00-0x3F), extending the array. */
  function setPaletteEntry(src, i, value) {
    if (!(value >= 0 && value <= 0x3f && value === (value | 0))) throw new Error("NES colours are 0x00-0x3F");
    if (!(i >= 0 && i <= 31)) throw new Error("a palette has 32 entries (0-31)");
    const p = findPalette(src);
    if (!p) throw new Error("no palette: call load_palette(NAME) with a const [u8] array");
    if (i < p.entries.length) {
      const e = p.entries[i];
      if (e.value === null) throw new Error(`entry ${i} is an expression (${e.text}); edit it in the code`);
      return replaceAll(src, [{ start: e.start, end: e.end, text: hex(value) }]);
    }
    const added = [];
    for (let k = p.entries.length; k <= i; k++) added.push(k === i ? hex(value) : "0x0F");
    if (p.entries.length === 0) return replaceAll(src, [{ start: p.open + 1, end: p.open + 1, text: added.join(", ") }]);
    const last = p.entries[p.entries.length - 1];
    return replaceAll(src, [{ start: last.end, end: last.end, text: ", " + added.join(", ") }]);
  }

  // ------------------------------------------------------------------ words and completion

  function wordAt(text, pos) {
    let start = pos, end = pos;
    while (start > 0 && isWord(text[start - 1])) start--;
    while (end < text.length && isWord(text[end])) end++;
    if (start === end || /^[0-9]/.test(text[start])) return null;
    return { word: text.slice(start, end), start, end };
  }

  function prefixAt(text, pos) {
    let start = pos;
    while (start > 0 && isWord(text[start - 1])) start--;
    const p = text.slice(start, pos);
    return /^[0-9]/.test(p) ? "" : p;
  }

  /** Names declared in the source: functions (with parameters) and everything else. */
  function declaredNames(src) {
    const m = mask(src);
    const out = [];
    const re = /\b(fn|var|const|let|map|metasprite|music|sound|struct|palette_zone)\s+([A-Za-z_]\w*)\s*(\(([^)]*)\))?/g;
    let hit;
    while ((hit = re.exec(m))) {
      if (hit[1] === "fn") {
        const params = (hit[4] || "").trim();
        out.push({ label: hit[2], kind: "user-fn", detail: `${hit[2]}(${params})`, call: true, args: params !== "" });
      } else if (hit[1] !== "palette_zone") {
        out.push({ label: hit[2], kind: "user", detail: hit[1], call: false, args: false });
      }
    }
    return out;
  }

  /** Completion candidates for `prefix`: exact-case matches first, then shortest. */
  function complete(prefix, builtins, src) {
    if (!prefix) return [];
    const all = [];
    for (const f of builtins.functions || []) {
      all.push({ label: f.name, kind: "function", detail: f.sig, doc: f.doc, call: true, args: !/\(\s*\)/.test(f.sig) });
    }
    all.push(...declaredNames(src || ""));
    for (const c of builtins.constants || []) all.push({ label: c, kind: "constant", detail: "", call: false, args: false });
    for (const k of builtins.keywords || []) all.push({ label: k, kind: "keyword", detail: "", call: false, args: false });
    const seen = new Set();
    const low = prefix.toLowerCase();
    return all
      .filter((c) => c.label.toLowerCase().startsWith(low) && !seen.has(c.label) && seen.add(c.label))
      .sort((a, b) => {
        const ea = a.label.startsWith(prefix) ? 0 : 1, eb = b.label.startsWith(prefix) ? 0 : 1;
        return ea - eb || a.label.length - b.label.length || (a.label < b.label ? -1 : 1);
      });
  }

  // ------------------------------------------------------------------ ROM graphics

  /** CHR tiles of an iNES ROM as arrays of 64 colour numbers (0-3). */
  function decodeChr(rom) {
    const base = 16 + rom[4] * 16384;
    const count = (rom[5] * 8192) / 16;
    const tiles = [];
    for (let t = 0; t < count; t++) {
      const px = new Uint8Array(64);
      for (let r = 0; r < 8; r++) {
        const lo = rom[base + t * 16 + r], hi = rom[base + t * 16 + r + 8];
        for (let c = 0; c < 8; c++) {
          const b = 7 - c;
          px[r * 8 + c] = ((lo >> b) & 1) | (((hi >> b) & 1) << 1);
        }
      }
      tiles.push(px);
    }
    return tiles;
  }

  return {
    NES_RGB, mask, literal, findTiles, tileToCode, upsertTile, findMaps, setMapCell,
    findPalette, setPaletteEntry, wordAt, prefixAt, complete, decodeChr,
  };
});

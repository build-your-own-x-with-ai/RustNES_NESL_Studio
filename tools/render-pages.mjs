// Renders `docs/` — the static NESL Studio site for GitHub Pages.
//
// Three things make the page work off a plain file server under a subpath:
//   1. studio.js is inlined, so index.html is the whole editor
//   2. the wasm loader is injected, replacing the local `nesc serve` backend
//   3. every asset path stays relative, so /<repo>/ resolves correctly
//
// Usage: node tools/render-pages.mjs   (run tools/build-pages.sh to build first)

import { readFileSync, writeFileSync, mkdirSync, rmSync, cpSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const web = join(root, "crates/nesc/web");
const pkg = join(root, "crates/nesc-wasm/pkg");
const docs = join(root, "docs");

if (!existsSync(pkg)) {
  console.error("missing crates/nesc-wasm/pkg — run: wasm-pack build crates/nesc-wasm --release --target web");
  process.exit(1);
}

// Dynamic `import()` so a missing pkg/ degrades to the HTTP backend instead of
// killing the whole module script.
const LOADER = `<script type="module">
  (async () => {
    try {
      const m = await import("./pkg/nesc_wasm.js");
      await m.default();
      window.__nescReady({
        compile: m.nesc_compile,
        check: m.nesc_check,
        examples: m.nesc_examples,
        builtins: m.nesc_builtins,
      });
    } catch (e) {
      console.warn("nesc wasm unavailable, falling back to the HTTP API:", e);
      window.__nescReady(null);
    }
  })();
</script>`;

const studio = readFileSync(join(web, "studio.js"), "utf8");
const html = readFileSync(join(web, "index.html"), "utf8")
  .replace("<!--NESC_WASM_LOADER-->", LOADER)
  .replace('<script src="studio.js"></script>', `<script>\n${studio}\n</script>`);

for (const marker of ["<!--NESC_WASM_LOADER-->", 'src="studio.js"']) {
  if (html.includes(marker)) throw new Error(`render-pages: ${marker} was not substituted`);
}
if (html.includes("/api/compile") === false && !html.includes("call.compile")) {
  throw new Error("render-pages: index.html lost its compile backend");
}

rmSync(docs, { recursive: true, force: true });
mkdirSync(docs, { recursive: true });
cpSync(pkg, join(docs, "pkg"), { recursive: true });
// wasm-pack writes a `.gitignore` containing `*` into pkg/, which would hide
// the .wasm from git — the site is committed, so it has to go.
rmSync(join(docs, "pkg/.gitignore"), { force: true });
writeFileSync(join(docs, "index.html"), html);
writeFileSync(join(docs, ".nojekyll"), "");

console.log(`rendered docs/index.html (${(html.length / 1024).toFixed(0)} KB, studio.js inlined)`);

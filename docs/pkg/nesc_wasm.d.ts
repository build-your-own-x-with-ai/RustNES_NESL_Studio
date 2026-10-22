/* tslint:disable */
/* eslint-disable */

/**
 * `{"functions":[..],"constants":[..],"keywords":[..]}` for completion and hover.
 */
export function nesc_builtins(): string;

/**
 * `{"ok":true,"errors":[]}` or `{"ok":false,"errors":[..]}` — same compile, no ROM.
 */
export function nesc_check(src: string): string;

/**
 * `{"ok":true,"rom":"<base64>","prg_used":N,"asm":".."}` or `{"ok":false,"errors":[..]}`.
 */
export function nesc_compile(src: string): string;

/**
 * `[{"name":..,"source":..}, ..]` — the bundled example programs.
 */
export function nesc_examples(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly nesc_builtins: () => [number, number];
    readonly nesc_check: (a: number, b: number) => [number, number];
    readonly nesc_compile: (a: number, b: number) => [number, number];
    readonly nesc_examples: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

/* tslint:disable */
/* eslint-disable */

/**
 * Return the event log as JSON for the inspector debugger.
 */
export function inspector_get_event_log(): string;

/**
 * Return the network log as JSON for the inspector debugger.
 */
export function inspector_get_network_log(): string;

/**
 * Return all state variables as JSON for the inspector panel.
 */
export function inspector_get_state(): string;

/**
 * Return the node tree as JSON for the inspector panel.
 */
export function inspector_get_tree(): string;

/**
 * Hit-test at canvas coordinates and return node info as JSON.
 */
export function inspector_node_at(x: number, y: number): string;

/**
 * Set or clear the highlighted node path for the inspector overlay.
 */
export function inspector_set_highlight(path: string): void;

/**
 * Reload the app with new app_data without reinitializing WASM.
 * Used by the gallery to switch between examples.
 */
export function reset_and_reload(app_data: Uint8Array): void;

/**
 * Entry point called from JavaScript.
 * `app_data` is a binary-encoded RenderTree.
 * `canvas_id` is the HTML id of the canvas element to render into.
 */
export function start(app_data: Uint8Array, canvas_id: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly inspector_get_event_log: () => [number, number];
    readonly inspector_get_network_log: () => [number, number];
    readonly inspector_get_state: () => [number, number];
    readonly inspector_get_tree: () => [number, number];
    readonly inspector_node_at: (a: number, b: number) => [number, number];
    readonly inspector_set_highlight: (a: number, b: number) => void;
    readonly reset_and_reload: (a: number, b: number) => [number, number];
    readonly start: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasm_bindgen__closure__destroy__h2141726e50f10748: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hc393ba3cd147b0ee: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h0c730d0e3e2a348e: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hf407c1039d5710ce: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h80219dffc57f851b: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h99fb71317c0b1f25: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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

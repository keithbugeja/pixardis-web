/* tslint:disable */
/* eslint-disable */
export function compile_pixardis_source_with_errors(source: string): any;
export function compile_pixardis_source(source: string): string;
export function create_vm(width: number, height: number): WebVM;
export function step_vm(vm: WebVM, steps: number): void;
export function get_vm_framebuffer(vm: WebVM): Uint8Array;
export function load_vm_program(vm: WebVM, assembly: string): void;
export class WebVM {
  free(): void;
  constructor(width: number, height: number);
  load_program(assembly: string): void;
  step(steps: number): void;
  get_framebuffer(): Uint8Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly compile_pixardis_source_with_errors: (a: number, b: number) => any;
  readonly compile_pixardis_source: (a: number, b: number) => [number, number, number, number];
  readonly __wbg_webvm_free: (a: number, b: number) => void;
  readonly webvm_get_framebuffer: (a: number) => [number, number];
  readonly create_vm: (a: number, b: number) => number;
  readonly step_vm: (a: number, b: number) => void;
  readonly get_vm_framebuffer: (a: number) => [number, number];
  readonly load_vm_program: (a: number, b: number, c: number) => void;
  readonly webvm_load_program: (a: number, b: number, c: number) => void;
  readonly webvm_new: (a: number, b: number) => number;
  readonly webvm_step: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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

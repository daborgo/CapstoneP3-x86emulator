/* tslint:disable */
/* eslint-disable */
/**
 * Main Emulator structure exposed to JavaScript
 *
 * This struct combines the CPU and Memory and provides
 * the public API for the frontend.
 */
export class Emulator {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new emulator instance
   */
  constructor();
  /**
   * Load a program's raw bytes into memory at a given address and set EIP there
   * Returns Err(String) on memory write failure.
   */
  load_program(program: Uint8Array, load_address: number): void;
  /**
   * Execute one instruction using fetch-decode-execute cycle
   *
   * This implements the complete CPU cycle:
   * 1. FETCH: Read instruction bytes from memory at EIP
   * 2. DECODE: Parse bytes into structured instruction
   * 3. EXECUTE: Execute the instruction
   * 4. Update step counter
   */
  step(): bigint;
  /**
   * Get the number of steps executed
   */
  get_steps(): bigint;
  /**
   * Get EAX register value (for testing)
   */
  get_eax(): number;
  /**
   * Additional register getters useful for UI
   */
  get_ebx(): number;
  get_ecx(): number;
  get_edx(): number;
  get_ebp(): number;
  get_esi(): number;
  get_edi(): number;
  /**
   * Set EAX register value (for testing)
   */
  set_eax(value: number): void;
  /**
   * Get EIP (instruction pointer) value
   */
  get_eip(): number;
  /**
   * Get ESP (stack pointer) value
   */
  get_esp(): number;
  /**
   * Flag getters
   */
  get_cf(): boolean;
  get_pf(): boolean;
  get_af(): boolean;
  get_zf(): boolean;
  get_sf(): boolean;
  get_of(): boolean;
  /**
   * Reset the emulator to initial state
   */
  reset(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_emulator_free: (a: number, b: number) => void;
  readonly emulator_new: () => number;
  readonly emulator_load_program: (a: number, b: number, c: number, d: number) => [number, number];
  readonly emulator_step: (a: number) => bigint;
  readonly emulator_get_steps: (a: number) => bigint;
  readonly emulator_get_eax: (a: number) => number;
  readonly emulator_get_ebx: (a: number) => number;
  readonly emulator_get_ecx: (a: number) => number;
  readonly emulator_get_edx: (a: number) => number;
  readonly emulator_get_ebp: (a: number) => number;
  readonly emulator_get_esi: (a: number) => number;
  readonly emulator_get_edi: (a: number) => number;
  readonly emulator_set_eax: (a: number, b: number) => void;
  readonly emulator_get_eip: (a: number) => number;
  readonly emulator_get_esp: (a: number) => number;
  readonly emulator_get_cf: (a: number) => number;
  readonly emulator_get_pf: (a: number) => number;
  readonly emulator_get_af: (a: number) => number;
  readonly emulator_get_zf: (a: number) => number;
  readonly emulator_get_sf: (a: number) => number;
  readonly emulator_get_of: (a: number) => number;
  readonly emulator_reset: (a: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
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

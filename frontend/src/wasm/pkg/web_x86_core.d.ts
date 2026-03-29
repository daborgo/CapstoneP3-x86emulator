/* tslint:disable */
/* eslint-disable */
export function grade_lab(lab_id: number, program: Uint8Array): string;
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
   * Load a program's raw bytes into memory at a given address and set EIP there
   * Returns Err(String) on memory write failure.
   */
  load_program(program: Uint8Array, load_address: number): void;
  /**
   * Create a new emulator instance
   */
  constructor();
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
   * Reset the emulator to initial state
   */
  reset(): void;
  get_af(): boolean;
  /**
   * Flag getters
   */
  get_cf(): boolean;
  get_of(): boolean;
  get_pf(): boolean;
  get_sf(): boolean;
  get_zf(): boolean;
  /**
   * Get EAX register value (for testing)
   */
  get_eax(): number;
  get_ebp(): number;
  /**
   * Additional register getters useful for UI
   */
  get_ebx(): number;
  get_ecx(): number;
  get_edi(): number;
  get_edx(): number;
  /**
   * Get EIP (instruction pointer) value
   */
  get_eip(): number;
  get_esi(): number;
  /**
   * Get ESP (stack pointer) value
   */
  get_esp(): number;
  /**
   * Read a single byte from memory (used by the UI memory viewer)
   */
  read_u8(addr: number): number;
  /**
   * Set EAX register value (for testing)
   */
  set_eax(value: number): void;
  /**
   * Read a 32-bit value from memory (for result checking / grading)
   */
  read_u32(addr: number): number;
  /**
   * Get the number of steps executed
   */
  get_steps(): bigint;
  /**
   * Write a 32-bit value to memory (for test setup / grading)
   */
  write_u32(addr: number, val: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly grade_lab: (a: number, b: number, c: number) => [number, number];
  readonly __wbg_emulator_free: (a: number, b: number) => void;
  readonly emulator_get_af: (a: number) => number;
  readonly emulator_get_cf: (a: number) => number;
  readonly emulator_get_eax: (a: number) => number;
  readonly emulator_get_ebp: (a: number) => number;
  readonly emulator_get_ebx: (a: number) => number;
  readonly emulator_get_ecx: (a: number) => number;
  readonly emulator_get_edi: (a: number) => number;
  readonly emulator_get_edx: (a: number) => number;
  readonly emulator_get_eip: (a: number) => number;
  readonly emulator_get_esi: (a: number) => number;
  readonly emulator_get_esp: (a: number) => number;
  readonly emulator_get_of: (a: number) => number;
  readonly emulator_get_pf: (a: number) => number;
  readonly emulator_get_sf: (a: number) => number;
  readonly emulator_get_steps: (a: number) => bigint;
  readonly emulator_get_zf: (a: number) => number;
  readonly emulator_load_program: (a: number, b: number, c: number, d: number) => [number, number];
  readonly emulator_new: () => number;
  readonly emulator_read_u32: (a: number, b: number) => [number, number, number];
  readonly emulator_read_u8: (a: number, b: number) => [number, number, number];
  readonly emulator_reset: (a: number) => void;
  readonly emulator_set_eax: (a: number, b: number) => void;
  readonly emulator_step: (a: number) => bigint;
  readonly emulator_write_u32: (a: number, b: number, c: number) => [number, number];
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
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

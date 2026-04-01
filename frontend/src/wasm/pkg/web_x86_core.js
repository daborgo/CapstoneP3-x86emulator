let wasm;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
 * @param {number} lab_id
 * @param {Uint8Array} program
 * @returns {string}
 */
export function grade_lab(lab_id, program) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passArray8ToWasm0(program, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.grade_lab(lab_id, ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

const EmulatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_emulator_free(ptr >>> 0, 1));
/**
 * Main Emulator structure exposed to JavaScript
 *
 * This struct combines the CPU and Memory and provides
 * the public API for the frontend.
 */
export class Emulator {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EmulatorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_emulator_free(ptr, 0);
    }
    /**
     * Load a program's raw bytes into memory at a given address and set EIP there
     * Returns Err(String) on memory write failure.
     * @param {Uint8Array} program
     * @param {number} load_address
     */
    load_program(program, load_address) {
        const ptr0 = passArray8ToWasm0(program, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.emulator_load_program(this.__wbg_ptr, ptr0, len0, load_address);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Create a new emulator instance
     */
    constructor() {
        const ret = wasm.emulator_new();
        this.__wbg_ptr = ret >>> 0;
        EmulatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Execute one instruction using fetch-decode-execute cycle
     *
     * This implements the complete CPU cycle:
     * 1. FETCH: Read instruction bytes from memory at EIP
     * 2. DECODE: Parse bytes into structured instruction
     * 3. EXECUTE: Execute the instruction
     * 4. Update step counter
     * @returns {bigint}
     */
    step() {
        const ret = wasm.emulator_step(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Reset the emulator to initial state
     */
    reset() {
        wasm.emulator_reset(this.__wbg_ptr);
    }
    /**
     * @returns {boolean}
     */
    get_af() {
        const ret = wasm.emulator_get_af(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Flag getters
     * @returns {boolean}
     */
    get_cf() {
        const ret = wasm.emulator_get_cf(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get_of() {
        const ret = wasm.emulator_get_of(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get_pf() {
        const ret = wasm.emulator_get_pf(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get_sf() {
        const ret = wasm.emulator_get_sf(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get_zf() {
        const ret = wasm.emulator_get_zf(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get EAX register value (for testing)
     * @returns {number}
     */
    get_eax() {
        const ret = wasm.emulator_get_eax(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_ebp() {
        const ret = wasm.emulator_get_ebp(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Additional register getters useful for UI
     * @returns {number}
     */
    get_ebx() {
        const ret = wasm.emulator_get_ebx(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_ecx() {
        const ret = wasm.emulator_get_ecx(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_edi() {
        const ret = wasm.emulator_get_edi(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_edx() {
        const ret = wasm.emulator_get_edx(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Set EBX register value
     * @param {number} value
     */
    set_ebx(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_ebx(this.__wbg_ptr, value);
    }
    /**
     * Set ECX register value
     * @param {number} value
     */
    set_ecx(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_ecx(this.__wbg_ptr, value);
    }
    /**
     * Set EDX register value
     * @param {number} value
     */
    set_edx(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_edx(this.__wbg_ptr, value);
    }
    /**
     * Set EBP register value
     * @param {number} value
     */
    set_ebp(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_ebp(this.__wbg_ptr, value);
    }
    /**
     * Set ESP register value
     * @param {number} value
     */
    set_esp(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_esp(this.__wbg_ptr, value);
    }
    /**
     * Set ESI register value
     * @param {number} value
     */
    set_esi(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_esi(this.__wbg_ptr, value);
    }
    /**
     * Set EDI register value
     * @param {number} value
     */
    set_edi(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_edi(this.__wbg_ptr, value);
    }
    /**
     * Set EIP (instruction pointer) value
     * @param {number} value
     */
    set_eip(value) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(value);
        wasm.emulator_set_eip(this.__wbg_ptr, value);
    }
    /**
     * Get EIP (instruction pointer) value
     * @returns {number}
     */
    get_eip() {
        const ret = wasm.emulator_get_eip(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_esi() {
        const ret = wasm.emulator_get_esi(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get ESP (stack pointer) value
     * @returns {number}
     */
    get_esp() {
        const ret = wasm.emulator_get_esp(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Read a single byte from memory (used by the UI memory viewer)
     * @param {number} addr
     * @returns {number}
     */
    read_u8(addr) {
        const ret = wasm.emulator_read_u8(this.__wbg_ptr, addr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Set EAX register value (for testing)
     * @param {number} value
     */
    set_eax(value) {
        wasm.emulator_set_eax(this.__wbg_ptr, value);
    }
    /**
     * Read a 32-bit value from memory (for result checking / grading)
     * @param {number} addr
     * @returns {number}
     */
    read_u32(addr) {
        const ret = wasm.emulator_read_u32(this.__wbg_ptr, addr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Get the number of steps executed
     * @returns {bigint}
     */
    get_steps() {
        const ret = wasm.emulator_get_steps(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Write a 32-bit value to memory (for test setup / grading)
     * @param {number} addr
     * @param {number} val
     */
    write_u32(addr, val) {
        const ret = wasm.emulator_write_u32(this.__wbg_ptr, addr, val);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) Emulator.prototype[Symbol.dispose] = Emulator.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg___wbindgen_throw_b855445ff6a94295 = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('web_x86_core_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;

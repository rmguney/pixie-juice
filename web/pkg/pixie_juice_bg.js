let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export_0(addHeapObject(e));
    }
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function pixie_optimize_auto(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.pixie_optimize_auto(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function pixie_optimize_image(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.pixie_optimize_image(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {Uint8Array}
 */
export function pixie_optimize_mesh(data) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.pixie_optimize_mesh(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @returns {number}
 */
export function pixie_get_memory_target_mb() {
    const ret = wasm.pixie_get_memory_target_mb();
    return ret;
}

/**
 * @returns {any}
 */
export function pixie_get_performance_stats() {
    const ret = wasm.get_performance_metrics();
    return takeObject(ret);
}

export function pixie_reset_performance_stats() {
    wasm.pixie_reset_performance_stats();
}

/**
 * @returns {boolean}
 */
export function pixie_check_performance_compliance() {
    const ret = wasm.check_performance_compliance();
    return ret !== 0;
}

/**
 * @returns {any}
 */
export function run_wasm_benchmarks() {
    const ret = wasm.run_wasm_benchmarks();
    return takeObject(ret);
}

export function init() {
    wasm.init();
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_image(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_image(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number | null} [target_ratio]
 * @returns {Uint8Array}
 */
export function optimize_mesh(data, target_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_mesh(retptr, ptr0, len0, isLikeNone(target_ratio) ? 0x100000001 : Math.fround(target_ratio));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_auto(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_auto(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.version(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export_1(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {string}
 */
export function build_timestamp() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.build_timestamp(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export_1(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {string}
 */
export function detect_format(data) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.detect_format(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export_1(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @returns {any}
 */
export function get_performance_metrics() {
    const ret = wasm.get_performance_metrics();
    return takeObject(ret);
}

export function reset_performance_stats() {
    wasm.pixie_reset_performance_stats();
}

/**
 * @returns {boolean}
 */
export function check_performance_compliance() {
    const ret = wasm.check_performance_compliance();
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_png(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_png(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_jpeg(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_jpeg(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_webp(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_webp(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_gif(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_gif(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_ico(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_ico(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function optimize_tga(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_tga(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_webp(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_webp(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_gif(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_gif(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_ico(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_ico(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_tga(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_tga(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_webp(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_webp(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {Uint8Array}
 */
export function convert_to_png(data) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_png(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_jpeg(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_jpeg(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {Uint8Array}
 */
export function convert_to_bmp(data) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_bmp(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_gif(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_gif(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_ico(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_ico(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_tiff(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_tiff(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {boolean} preserve_icc
 * @returns {Uint8Array}
 */
export function strip_tiff_metadata_simd(data, preserve_icc) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.strip_tiff_metadata_simd(retptr, ptr0, len0, preserve_icc);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_svg(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_svg(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} quality
 * @returns {Uint8Array}
 */
export function convert_to_tga(data, quality) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.convert_to_tga(retptr, ptr0, len0, quality);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {boolean} enabled
 * @returns {any}
 */
export function set_lossless_mode(enabled) {
    const ret = wasm.set_lossless_mode(enabled);
    return takeObject(ret);
}

/**
 * @param {boolean} enabled
 * @returns {any}
 */
export function set_preserve_metadata(enabled) {
    const ret = wasm.set_preserve_metadata(enabled);
    return takeObject(ret);
}

/**
 * @param {Uint8Array} data
 * @param {number} reduction_ratio
 * @returns {Uint8Array}
 */
export function optimize_obj(data, reduction_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_obj(retptr, ptr0, len0, reduction_ratio);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} reduction_ratio
 * @returns {Uint8Array}
 */
export function optimize_gltf(data, reduction_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_gltf(retptr, ptr0, len0, reduction_ratio);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} reduction_ratio
 * @returns {Uint8Array}
 */
export function optimize_stl(data, reduction_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_stl(retptr, ptr0, len0, reduction_ratio);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} reduction_ratio
 * @returns {Uint8Array}
 */
export function optimize_fbx(data, reduction_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_fbx(retptr, ptr0, len0, reduction_ratio);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @param {number} reduction_ratio
 * @returns {Uint8Array}
 */
export function optimize_ply(data, reduction_ratio) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
        const len0 = WASM_VECTOR_LEN;
        wasm.optimize_ply(retptr, ptr0, len0, reduction_ratio);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v2 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export_1(r0, r1 * 1, 1);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_obj(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_obj(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_gltf(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_gltf(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_stl(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_stl(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_fbx(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_fbx(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {Uint8Array} data
 * @returns {boolean}
 */
export function is_ply(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_ply(ptr0, len0);
    return ret !== 0;
}

/**
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7}
 */
export const ColorSpace = Object.freeze({
    RGB: 0, "0": "RGB",
    RGBA: 1, "1": "RGBA",
    Grayscale: 2, "2": "Grayscale",
    GrayscaleAlpha: 3, "3": "GrayscaleAlpha",
    CMYK: 4, "4": "CMYK",
    YUV: 5, "5": "YUV",
    HSV: 6, "6": "HSV",
    LAB: 7, "7": "LAB",
});
/**
 * @enum {0 | 1 | 2}
 */
export const SimplificationAlgorithm = Object.freeze({
    QuadricErrorMetrics: 0, "0": "QuadricErrorMetrics",
    EdgeCollapse: 1, "1": "EdgeCollapse",
    VertexClustering: 2, "2": "VertexClustering",
});

const ImageOptConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imageoptconfig_free(ptr >>> 0, 1));

export class ImageOptConfig {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageOptConfig.prototype);
        obj.__wbg_ptr = ptr;
        ImageOptConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageOptConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imageoptconfig_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get quality() {
        const ret = wasm.__wbg_get_imageoptconfig_quality(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set quality(arg0) {
        wasm.__wbg_set_imageoptconfig_quality(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get lossless() {
        const ret = wasm.__wbg_get_imageoptconfig_lossless(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set lossless(arg0) {
        wasm.__wbg_set_imageoptconfig_lossless(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get preserve_metadata() {
        const ret = wasm.__wbg_get_imageoptconfig_preserve_metadata(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set preserve_metadata(arg0) {
        wasm.__wbg_set_imageoptconfig_preserve_metadata(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get optimize_colors() {
        const ret = wasm.__wbg_get_imageoptconfig_optimize_colors(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set optimize_colors(arg0) {
        wasm.__wbg_set_imageoptconfig_optimize_colors(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number | undefined}
     */
    get max_colors() {
        const ret = wasm.__wbg_get_imageoptconfig_max_colors(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * @param {number | null} [arg0]
     */
    set max_colors(arg0) {
        wasm.__wbg_set_imageoptconfig_max_colors(this.__wbg_ptr, isLikeNone(arg0) ? 0xFFFFFF : arg0);
    }
    /**
     * @returns {boolean}
     */
    get use_c_hotspots() {
        const ret = wasm.__wbg_get_imageoptconfig_use_c_hotspots(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set use_c_hotspots(arg0) {
        wasm.__wbg_set_imageoptconfig_use_c_hotspots(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get enable_simd() {
        const ret = wasm.__wbg_get_imageoptconfig_enable_simd(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set enable_simd(arg0) {
        wasm.__wbg_set_imageoptconfig_enable_simd(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number | undefined}
     */
    get compression_level() {
        const ret = wasm.__wbg_get_imageoptconfig_compression_level(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * @param {number | null} [arg0]
     */
    set compression_level(arg0) {
        wasm.__wbg_set_imageoptconfig_compression_level(this.__wbg_ptr, isLikeNone(arg0) ? 0xFFFFFF : arg0);
    }
    /**
     * @returns {boolean}
     */
    get fast_mode() {
        const ret = wasm.__wbg_get_imageoptconfig_fast_mode(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set fast_mode(arg0) {
        wasm.__wbg_set_imageoptconfig_fast_mode(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get preserve_alpha() {
        const ret = wasm.__wbg_get_imageoptconfig_preserve_alpha(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set preserve_alpha(arg0) {
        wasm.__wbg_set_imageoptconfig_preserve_alpha(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number | undefined}
     */
    get max_width() {
        const ret = wasm.__wbg_get_imageoptconfig_max_width(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [arg0]
     */
    set max_width(arg0) {
        wasm.__wbg_set_imageoptconfig_max_width(this.__wbg_ptr, isLikeNone(arg0) ? 0x100000001 : (arg0) >>> 0);
    }
    /**
     * @returns {number | undefined}
     */
    get max_height() {
        const ret = wasm.__wbg_get_imageoptconfig_max_height(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [arg0]
     */
    set max_height(arg0) {
        wasm.__wbg_set_imageoptconfig_max_height(this.__wbg_ptr, isLikeNone(arg0) ? 0x100000001 : (arg0) >>> 0);
    }
    /**
     * @returns {number | undefined}
     */
    get target_reduction() {
        const ret = wasm.__wbg_get_imageoptconfig_target_reduction(this.__wbg_ptr);
        return ret === 0x100000001 ? undefined : ret;
    }
    /**
     * @param {number | null} [arg0]
     */
    set target_reduction(arg0) {
        wasm.__wbg_set_imageoptconfig_target_reduction(this.__wbg_ptr, isLikeNone(arg0) ? 0x100000001 : Math.fround(arg0));
    }
}

const MeshOptConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_meshoptconfig_free(ptr >>> 0, 1));

export class MeshOptConfig {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MeshOptConfig.prototype);
        obj.__wbg_ptr = ptr;
        MeshOptConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MeshOptConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_meshoptconfig_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get target_ratio() {
        const ret = wasm.__wbg_get_meshoptconfig_target_ratio(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set target_ratio(arg0) {
        wasm.__wbg_set_meshoptconfig_target_ratio(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get preserve_topology() {
        const ret = wasm.__wbg_get_meshoptconfig_preserve_topology(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set preserve_topology(arg0) {
        wasm.__wbg_set_meshoptconfig_preserve_topology(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get weld_vertices() {
        const ret = wasm.__wbg_get_meshoptconfig_weld_vertices(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set weld_vertices(arg0) {
        wasm.__wbg_set_meshoptconfig_weld_vertices(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get vertex_tolerance() {
        const ret = wasm.__wbg_get_meshoptconfig_vertex_tolerance(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set vertex_tolerance(arg0) {
        wasm.__wbg_set_meshoptconfig_vertex_tolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {SimplificationAlgorithm}
     */
    get simplification_algorithm() {
        const ret = wasm.__wbg_get_meshoptconfig_simplification_algorithm(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {SimplificationAlgorithm} arg0
     */
    set simplification_algorithm(arg0) {
        wasm.__wbg_set_meshoptconfig_simplification_algorithm(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get use_c_hotspots() {
        const ret = wasm.__wbg_get_meshoptconfig_use_c_hotspots(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set use_c_hotspots(arg0) {
        wasm.__wbg_set_meshoptconfig_use_c_hotspots(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get generate_normals() {
        const ret = wasm.__wbg_get_meshoptconfig_generate_normals(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set generate_normals(arg0) {
        wasm.__wbg_set_meshoptconfig_generate_normals(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get optimize_vertex_cache() {
        const ret = wasm.__wbg_get_meshoptconfig_optimize_vertex_cache(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set optimize_vertex_cache(arg0) {
        wasm.__wbg_set_meshoptconfig_optimize_vertex_cache(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get preserve_uv_seams() {
        const ret = wasm.__wbg_get_meshoptconfig_preserve_uv_seams(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set preserve_uv_seams(arg0) {
        wasm.__wbg_set_meshoptconfig_preserve_uv_seams(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get preserve_boundaries() {
        const ret = wasm.__wbg_get_meshoptconfig_preserve_boundaries(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set preserve_boundaries(arg0) {
        wasm.__wbg_set_meshoptconfig_preserve_boundaries(this.__wbg_ptr, arg0);
    }
}

const PixieConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pixieconfig_free(ptr >>> 0, 1));

export class PixieConfig {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PixieConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pixieconfig_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.pixieconfig_new();
        this.__wbg_ptr = ret >>> 0;
        PixieConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {boolean}
     */
    get use_c_hotspots() {
        const ret = wasm.pixieconfig_use_c_hotspots(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} value
     */
    set use_c_hotspots(value) {
        wasm.pixieconfig_set_use_c_hotspots(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    get quality() {
        const ret = wasm.pixieconfig_quality(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} quality
     */
    set quality(quality) {
        wasm.pixieconfig_set_quality(this.__wbg_ptr, quality);
    }
    /**
     * @returns {boolean}
     */
    get enable_threading() {
        const ret = wasm.pixieconfig_enable_threading(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} value
     */
    set enable_threading(value) {
        wasm.pixieconfig_set_enable_threading(this.__wbg_ptr, value);
    }
    /**
     * @returns {ImageOptConfig}
     */
    to_image_config() {
        const ret = wasm.pixieconfig_to_image_config(this.__wbg_ptr);
        return ImageOptConfig.__wrap(ret);
    }
    /**
     * @returns {MeshOptConfig}
     */
    to_mesh_config() {
        const ret = wasm.pixieconfig_to_mesh_config(this.__wbg_ptr);
        return MeshOptConfig.__wrap(ret);
    }
}

export function __wbg_call_672a4d21634d4a24() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_error_7534b8e9a36f1ab4(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_export_1(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_instanceof_Window_def73ea0955fc569(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_log_0cc1b7768397bcfe(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
    } finally {
        wasm.__wbindgen_export_1(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_log_8a185c59294f9f1f(arg0, arg1) {
    console.log(getStringFromWasm0(arg0, arg1));
};

export function __wbg_log_c222819a41e063d3(arg0) {
    console.log(getObject(arg0));
};

export function __wbg_log_cb9e190acc5753fb(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.log(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_export_1(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_mark_7438147ce31e9d4b(arg0, arg1) {
    performance.mark(getStringFromWasm0(arg0, arg1));
};

export function __wbg_measure_fb7825c11612c823() { return handleError(function (arg0, arg1, arg2, arg3) {
    let deferred0_0;
    let deferred0_1;
    let deferred1_0;
    let deferred1_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        deferred1_0 = arg2;
        deferred1_1 = arg3;
        performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
    } finally {
        wasm.__wbindgen_export_1(deferred0_0, deferred0_1, 1);
        wasm.__wbindgen_export_1(deferred1_0, deferred1_1, 1);
    }
}, arguments) };

export function __wbg_new_405e22f390576ce2() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_new_78feb108b6472713() {
    const ret = new Array();
    return addHeapObject(ret);
};

export function __wbg_new_8a6f238a6ece86ea() {
    const ret = new Error();
    return addHeapObject(ret);
};

export function __wbg_newnoargs_105ed471475aaf50(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_now_7f43f6c42e10de17() {
    const ret = performance.now();
    return ret;
};

export function __wbg_now_d18023d54d4e5500(arg0) {
    const ret = getObject(arg0).now();
    return ret;
};

export function __wbg_performance_c185c0cdc2766575(arg0) {
    const ret = getObject(arg0).performance;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_set_37837023f3d740e8(arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
};

export function __wbg_set_3f1d0b984ed272ed(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
};

export function __wbg_stack_0ed75d68575b0f3c(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_2, wasm.__wbindgen_export_3);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_static_accessor_GLOBAL_88a902d13a557d07() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_static_accessor_SELF_37c5d418e4bf5819() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_static_accessor_WINDOW_5de37043a91a9c40() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbindgen_bigint_from_u64(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
};

export function __wbindgen_error_new(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};


let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

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

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getObject(idx) { return heap[idx]; }

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

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(takeObject(mem.getUint32(i, true)));
    }
    return result;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

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
/**
 * Gathers valid words for a given 12-letter input, returning them in serialized form.
 *
 * # Panics
 *
 * Panics if the letter sequences cannot be serialized.
 * @param {string} input
 * @returns {SerializedSequences}
 */
export function getValidWords(input) {
    const ptr0 = passStringToWasm0(input, wasm.__wbindgen_export_1, wasm.__wbindgen_export_2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.getValidWords(ptr0, len0);
    return SerializedSequences.__wrap(ret);
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
 * Deserializes and stores valid words in thread-local storage for later use.
 * Solutions are generated in chunks, so this vector is reused multiple times.
 *
 * # Panics
 *
 * Panics if the serialized words cannot be deserialized.
 * @param {Uint8Array} serialized_words
 */
export function registerValidWords(serialized_words) {
    const ptr0 = passArray8ToWasm0(serialized_words, wasm.__wbindgen_export_1);
    const len0 = WASM_VECTOR_LEN;
    wasm.registerValidWords(ptr0, len0);
}

/**
 * Clears the currently registered valid words from thread-local storage.
 */
export function clearValidWords() {
    wasm.clearValidWords();
}

/**
 * Generates puzzle solutions for valid words in the specified index range.
 * @param {number} range_start
 * @param {number} range_end
 * @returns {SolutionsPayload}
 */
export function solutions(range_start, range_end) {
    const ret = wasm.solutions(range_start, range_end);
    return SolutionsPayload.__wrap(ret);
}

const LetterSequenceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_lettersequence_free(ptr >>> 0, 1));
/**
 * [`LetterSequence`] is a stack-allocated vector of up to 12 uppercase [ASCII] letters represented internally by
 * a single [u64] value.
 *
 * Since there are 26 letters in the English alphabet, each letter can be represented
 * uniquely with only 5 bits of data by subtracting the [ASCII] value for `'A'` from each letter.
 *
 * * `'A'` is represented by `00000`
 * * `'B'` is represented by `00001`
 * * `'C'` is represented by `00010`
 * * `...`
 * * `'X'` is represented by `10111`
 * * `'Y'` is represented by `11000`
 * * `'Z'` is represented by `11001`
 *
 * We can divide the [u64] into 12 sections of 5 bits, fitting up to 12 [ASCII] letters, with 4 extra bits left over.
 *
 * One of the 4 extra bits is used to retain track of count of letters in the [`LetterSequence`] by maintaining a single
 * one-bit that separates not-yet-filled data from populated data.
 *
 * The internal representation of the 64 bits within an empty [`LetterSequence`] will look like this:
 *
 * ```text
 *                                                         Length-tracker bit ╾┐
 *                                                                             │
 * 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
 * └┬┘ └──────────────────────────────────┬──────────────────────────────────┘
 *  └╼ Extra unused bits                  └╼ Empty letter space
 * ```
 *
 * Consider an example where the letter `'A'` is appended to the empty [`LetterSequence`] shown above.
 *
 * The [ASCII] value for `'A'` is `1000001`. This [ASCII] value will be shifted to match the 5-bit
 * representation described above, making its value equal to `00000`. It will then be appended
 * to the [`LetterSequence`], shifting the position of the length-tracker bit by 5 bits as well:
 *
 * ```text
 *                                                         Length-tracker bit ╾┐
 *                                                                             │
 * 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
 * └┬┘ └──────────────────────────────────┬──────────────────────────────────┘ │
 *  └╼ Extra unused bits                  └╼ Empty letter space          ┌─────┘
 *                                                                       │
 * 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000
 * └┬┘ └───────────────────────────────┬───────────────────────────────┘   │ A │
 *  └╼ Extra unused bits               └╼ Empty letter space               └───┘
 * ```
 *
 * Note that the length-tracker bit is critical for knowing that the group of `00000` to the right
 * of the bit is the letter `'A'`, whereas the group of `00000` to the left of the bit is empty space.
 *
 * Now consider appending the letter `'F'` to the same [`LetterSequence`] that we just appended `'A'` to.
 *
 * The [ASCII] value for `'F'` is `1000110`. This [ASCII] value will be shifted to match the 5-bit
 * representation described above, making its value equal to `00101`. It will then be appended
 * to the [`LetterSequence`], shifting the position of the length-tracker bit by 5 bits as well:
 *
 * ```text
 *                                                   Length-tracker bit ╾┐
 *                                                                       │
 * 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000
 * └┬┘ └───────────────────────────────┬───────────────────────────────┘ │ │ A │
 *  └╼ Extra unused bits               └╼ Empty letter space       ┌─────┘ └─┬─┘
 *                                                                 │   ┌─────┘
 *                                                                 │ ┌─┴─┐
 * 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000 00101
 * └┬┘ └────────────────────────────┬────────────────────────────┘   │ A │ │ F │
 *  └╼ Extra unused bits            └╼ Empty letter space            └───┘ └───┘
 * ```
 *
 * [ASCII]: https://en.wikipedia.org/wiki/ASCII
 */
export class LetterSequence {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LetterSequenceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_lettersequence_free(ptr, 0);
    }
}

const SerializedSequencesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_serializedsequences_free(ptr >>> 0, 1));
/**
 * A structure holding serialized words along with the total word count.
 */
export class SerializedSequences {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SerializedSequences.prototype);
        obj.__wbg_ptr = ptr;
        SerializedSequencesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SerializedSequencesFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_serializedsequences_free(ptr, 0);
    }
    /**
     * Returns the number of words in the serialized word list.
     * @returns {number}
     */
    get wordCount() {
        const ret = wasm.serializedsequences_wordCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the serialized list of valid words.
     * @returns {Uint8Array}
     */
    get serializedWords() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.serializedsequences_serializedWords(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

const SolutionsPayloadFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_solutionspayload_free(ptr >>> 0, 1));
/**
 * A payload to hold solution strings grouped by how many words are in the solution.
 * There must be at least 1 word in a solution, and there can be at most 5 words.
 */
export class SolutionsPayload {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SolutionsPayload.prototype);
        obj.__wbg_ptr = ptr;
        SolutionsPayloadFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SolutionsPayloadFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_solutionspayload_free(ptr, 0);
    }
    /**
     * Adds a [`LetterSequence`] solution to the relevant bucket based on the word count.
     * @param {LetterSequence} sequence
     */
    push(sequence) {
        _assertClass(sequence, LetterSequence);
        var ptr0 = sequence.__destroy_into_raw();
        wasm.solutionspayload_push(this.__wbg_ptr, ptr0);
    }
    /**
     * Takes and returns all one-word solutions, clearing them from the internal list.
     * @returns {string[]}
     */
    get oneWordSolutions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.solutionspayload_oneWordSolutions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Takes and returns all two-word solutions, clearing them from the internal list.
     * @returns {string[]}
     */
    get twoWordSolutions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.solutionspayload_twoWordSolutions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Takes and returns all three-word solutions, clearing them from the internal list.
     * @returns {string[]}
     */
    get threeWordSolutions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.solutionspayload_threeWordSolutions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Takes and returns all four-word solutions, clearing them from the internal list.
     * @returns {string[]}
     */
    get fourWordSolutions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.solutionspayload_fourWordSolutions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Takes and returns all five-word solutions, clearing them from the internal list.
     * @returns {string[]}
     */
    get fiveWordSolutions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.solutionspayload_fiveWordSolutions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export_0(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
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
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;



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

    __wbg_init_memory(imports);

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
        module_or_path = new URL('letrboxd_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;

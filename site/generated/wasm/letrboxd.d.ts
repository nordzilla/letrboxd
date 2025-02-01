/* tslint:disable */
/* eslint-disable */
/**
 * Gathers valid words for a given 12-letter input, returning them in serialized form.
 *
 * # Panics
 *
 * Panics if the letter sequences cannot be serialized.
 */
export function getValidWords(input: string): SerializedSequences;
/**
 * Deserializes and stores valid words in thread-local storage for later use.
 * Solutions are generated in chunks, so this vector is reused multiple times.
 *
 * # Panics
 *
 * Panics if the serialized words cannot be deserialized.
 */
export function registerValidWords(serialized_words: Uint8Array): void;
/**
 * Clears the currently registered valid words from thread-local storage.
 */
export function clearValidWords(): void;
/**
 * Generates puzzle solutions for valid words in the specified index range.
 */
export function solutions(range_start: number, range_end: number): SolutionsPayload;
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
  private constructor();
  free(): void;
}
/**
 * A structure holding serialized words along with the total word count.
 */
export class SerializedSequences {
  private constructor();
  free(): void;
  /**
   * Returns the number of words in the serialized word list.
   */
  readonly wordCount: number;
  /**
   * Returns the serialized list of valid words.
   */
  readonly serializedWords: Uint8Array;
}
/**
 * A payload to hold solution strings grouped by how many words are in the solution.
 * There must be at least 1 word in a solution, and there can be at most 5 words.
 */
export class SolutionsPayload {
  private constructor();
  free(): void;
  /**
   * Adds a [`LetterSequence`] solution to the relevant bucket based on the word count.
   */
  push(sequence: LetterSequence): void;
  /**
   * Takes and returns all one-word solutions, clearing them from the internal list.
   */
  readonly oneWordSolutions: string[];
  /**
   * Takes and returns all two-word solutions, clearing them from the internal list.
   */
  readonly twoWordSolutions: string[];
  /**
   * Takes and returns all three-word solutions, clearing them from the internal list.
   */
  readonly threeWordSolutions: string[];
  /**
   * Takes and returns all four-word solutions, clearing them from the internal list.
   */
  readonly fourWordSolutions: string[];
  /**
   * Takes and returns all five-word solutions, clearing them from the internal list.
   */
  readonly fiveWordSolutions: string[];
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_serializedsequences_free: (a: number, b: number) => void;
  readonly serializedsequences_wordCount: (a: number) => number;
  readonly serializedsequences_serializedWords: (a: number, b: number) => void;
  readonly __wbg_solutionspayload_free: (a: number, b: number) => void;
  readonly solutionspayload_push: (a: number, b: number) => void;
  readonly solutionspayload_oneWordSolutions: (a: number, b: number) => void;
  readonly solutionspayload_twoWordSolutions: (a: number, b: number) => void;
  readonly solutionspayload_threeWordSolutions: (a: number, b: number) => void;
  readonly solutionspayload_fourWordSolutions: (a: number, b: number) => void;
  readonly solutionspayload_fiveWordSolutions: (a: number, b: number) => void;
  readonly getValidWords: (a: number, b: number) => number;
  readonly registerValidWords: (a: number, b: number) => void;
  readonly clearValidWords: () => void;
  readonly solutions: (a: number, b: number) => number;
  readonly __wbg_lettersequence_free: (a: number, b: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number, d: number) => number;
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

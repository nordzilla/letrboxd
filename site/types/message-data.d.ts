/**
 * A collection of types to help define the data passed to and from workers.
 */

/**
 * Union type for all SolverWorker messages.
 */
export type SolverWorkerMessage =
  | InitializeWasmRequest
  | InitializeWasmResponse
  | ValidWordsRequest
  | ValidWordsResponse
  | SolutionsRequest
  | SolutionsResponse;

/**
 * Union type for all SorterWorker messages.
 */
export type SorterWorkerMessage =
  | SortedSolutionsRequest
  | SortedSolutionsResponse;

/**
 * Message for requesting that a worker initialize WASM using a provided binary.
 */
export interface InitializeWasmRequest {
  type: "InitializeWasmRequest";
  wasmBinary: ArrayBuffer;
}

/**
 * Message indicating WASM initialization.
 */
export interface InitializeWasmResponse {
  type: "WasmInitialized";
}

/**
 * Message for requesting valid words.
 */
export interface ValidWordsRequest {
  type: "ValidWordsRequest";
  requestId: number;
  text: string;
}

/**
 * Message for valid words payload.
 */
export interface ValidWordsResponse {
  type: "ValidWordsResponse";
  requestId: number;
  serializedWords: Uint8Array<ArrayBufferLike>;
  wordCount: number;
}

/**
 * Message for requesting solutions.
 */
export interface SolutionsRequest {
  type: "SolutionsRequest";
  requestId: number;
  serializedWords: Uint8Array<ArrayBufferLike>;
  rangeStart: number;
  rangeEnd: number;
}

/**
 * Message for solutions payload.
 */
export interface SolutionsResponse {
  type: "SolutionsResponse";
  requestId: number;
  isFinalResponse: boolean;
  oneWordSolutions: string[];
  twoWordSolutions: string[];
  threeWordSolutions: string[];
  fourWordSolutions: string[];
  fiveWordSolutions: string[];
}

/**
 * Message for sorting solutions.
 */
export interface SortedSolutionsRequest {
  type: "SortedSolutionsRequest";
  requestId: number;
  solutions: string[];
}

/**
 * Message for sorted solutions payload.
 */
export interface SortedSolutionsResponse {
  type: "SortedSolutionsResponse";
  requestId: number;
  sortedSolutions: string[];
}

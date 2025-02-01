/**
 * A Web Worker script that loads a WebAssembly module for LetrBoxd puzzle computations,
 * and handles messages from the main thread to process valid words or generate solutions.
 *
 * @typedef {import("./../types/message-data").InitializeWasmRequest} InitializeWasmRequest
 * @typedef {import("./../types/message-data").InitializeWasmResponse} InitializeWasmResponse
 * @typedef {import("./../types/message-data").ValidWordsRequest} ValidWordsRequest
 * @typedef {import("./../types/message-data").ValidWordsResponse} ValidWordsResponse
 * @typedef {import("./../types/message-data").SolutionsRequest} SolutionsRequest
 * @typedef {import("./../types/message-data").SolutionsResponse} SolutionsResponse
 * @typedef {import("./../types/message-data").SolverWorkerMessage} SolverWorkerMessage
 */

import * as wasm from "./../generated/wasm/letrboxd.js";

/**
 * Indicates whether the WASM module has completed its initialization.
 *
 * @type {boolean}
 */
let wasmInitialized = false;

/**
 * The ID of the most recent request handled by this worker.
 * This allows us to detect and ignore outdated requests if a newer one arrives.
 *
 * @type {number | null}
 */
let activeRequestId = null;

async function waitForTick() {
  const { promise, resolve } = Promise.withResolvers();
  setTimeout(resolve, 0);
  await promise;
}

/**
 * The main message handler for incoming requests from the main thread.
 * It routes messages based on their `type` property, and either provides valid words
 * or generates puzzle solutions in subranges, after ensuring the WASM module is initialized.
 *
 * @param {MessageEvent<SolverWorkerMessage>} event - The message event containing data from the main thread.
 */
self.onmessage = async (event) => {
  const { data } = event;
  const { type } = data;

  switch (type) {
    /**
     * Emitted by the main thread to provide the WASM binary and initialize the module
     * in this worker. Once initialized, the worker sends "WasmInitialized".
     *
     * @type {InitializeWasmRequest}
     */
    case "InitializeWasmRequest": {
      try {
        if (wasmInitialized) {
          // Already initialized; no need to re-initialize
          break;
        }

        const { wasmBinary } = data;
        wasm.initSync({ module: wasmBinary });
        wasmInitialized = true;

        /** @type {InitializeWasmResponse} */
        const initializedMessage = { type: "WasmInitialized" };
        self.postMessage(initializedMessage);
      } catch (error) {
        console.error("Failed to initialize WASM from binary:", error);
      }
      break;
    }

    /**
     * Emitted by the main thread to request a list of valid words for a given input text.
     * The worker responds with a "ValidWordsResponse" message, containing a serialized
     * list of valid words and a count of how many words are valid.
     *
     * @type {ValidWordsRequest}
     */
    case "ValidWordsRequest": {
      if (!wasmInitialized) {
        console.error("WASM module is not initialized");
        return;
      }

      const { requestId, text } = data;
      activeRequestId = requestId;

      const { wordCount, serializedWords } = wasm.getValidWords(text);

      /** @type {ValidWordsResponse} */
      const payload = {
        type: "ValidWordsResponse",
        requestId,
        wordCount,
        serializedWords,
      };

      self.postMessage(payload, undefined, [serializedWords]);
      break;
    }

    /**
     * Emitted by the main thread to request puzzle solutions for a particular range of words.
     * The worker splits this range into sections, processing each subrange and posting
     * "SolutionsResponse" messages back to the main thread. The final section is marked
     * with `isFinalResponse: true`.
     *
     * @type {SolutionsRequest}
     */
    case "SolutionsRequest": {
      if (!wasmInitialized) {
        console.error("WASM module is not initialized");
        return;
      }

      const { requestId, serializedWords, rangeStart, rangeEnd } = data;

      activeRequestId = requestId;
      wasm.registerValidWords(serializedWords);

      const range = rangeEnd - rangeStart;
      if (range <= 0) {
        console.error("Invalid range: rangeStart must be less than rangeEnd");
        return;
      }

      const maxSections = 4;
      const sections = Math.min(range, maxSections);
      const baseSize = Math.floor(range / sections);
      const extra = range % sections;

      let start = rangeStart;
      for (let index = 0; index < sections; index++) {
        const size = baseSize + (index < extra ? 1 : 0);
        const end = start + size;

        const {
          oneWordSolutions,
          twoWordSolutions,
          threeWordSolutions,
          fourWordSolutions,
          fiveWordSolutions,
        } = wasm.solutions(start, end);

        /** @type {SolutionsResponse} */
        const solutionsMessage = {
          type: "SolutionsResponse",
          requestId,
          oneWordSolutions,
          twoWordSolutions,
          threeWordSolutions,
          fourWordSolutions,
          fiveWordSolutions,
          isFinalResponse: index === sections - 1,
        };

        self.postMessage(solutionsMessage);

        await waitForTick();
        if (requestId !== activeRequestId) {
          // This request is no longer relevant.
          return;
        }

        start = end;
      }

      wasm.clearValidWords();
      break;
    }

    default: {
      throw new Error(`Unknown message received from main thread: ${type}`);
    }
  }
};

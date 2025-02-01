/**
 * The path to the solver worker script used to run computations in a worker thread.
 *
 * @type {string}
 */
const WORKER_PATH = "./site/workers/solver.worker.mjs";

/**
 * The path to the WASM binary to give provide solver workers.
 *
 * @type {string}
 */
const WASM_BINARY_PATH = "./site/generated/wasm/letrboxd_bg.wasm";

/**
 * Import type definitions for messages exchanged with the solver workers.
 *
 * @typedef {import("./types/wasm-worker").WasmWorker} WasmWorker
 * @typedef {import("./types/message-data").WorkerMessage} WorkerMessage
 * @typedef {import("./types/message-data").InitializeWasmRequest} InitializeWasmRequest
 * @typedef {import("./types/message-data").InitializeWasmResponse} InitializeWasmResponse
 * @typedef {import("./types/message-data").ValidWordsRequest} ValidWordsRequest
 * @typedef {import("./types/message-data").ValidWordsResponse} ValidWordsResponse
 * @typedef {import("./types/message-data").SolutionsRequest} SolutionsRequest
 * @typedef {import("./types/message-data").SolutionsResponse} SolutionsResponse
 */

/**
 * A pool of solver worker instances that handle puzzle computations in parallel.
 *
 * Each worker is responsible for running a portion of the puzzle logic in a dedicated thread,
 * ensuring that large computations do not block the main UI thread.
 */
export class SolverPool {
  /**
   * An array of `Worker` instances used by this solver pool.
   * Each worker runs a separate instance of the solver logic via the specified worker script.
   *
   * @type {WasmWorker[]}
   */
  #workers = [];

  /**
   * A collection of solution arrays, each index storing solutions of a particular word count.
   * Index 0 stores 1-word solutions, index 1 stores 2-word solutions, etc.
   *
   * @type {Solutions}
   */
  #solutions = [
    [], // 1-word solutions
    [], // 2-word solutions
    [], // 3-word solutions
    [], // 4-word solutions
    [], // 5-word solutions
  ];

  /**
   * The ID of the most recent request for which we are generating solutions.
   * Any responses from workers that do not match this request ID are ignored.
   *
   * @type {number | null}
   */
  #activeRequestId = null;

  /**
   * The count of workers that have signaled a final payload for the current request.
   *
   * @type {number}
   */
  #activeWorkerCount = 0;

  /**
   * Returns the default number of solvers to use in the SolverPool.
   * 
   * @returns {number}
   */
  static #defaultSolverCount() {
    return Math.min(16, navigator.hardwareConcurrency);
  }

  /**
   * Creates a new SolverPool instance, initializing up to `maxSolvers` worker threads,
   * fetching the WASM binary once, and distributing it to each worker.
   *
   * @param {number} [solverCount] - The number of worker threads to spawn.
   */
  constructor(solverCount = SolverPool.#defaultSolverCount()) {
    this.#workers = Array.from({ length: solverCount }, () =>
      new Worker(WORKER_PATH, { type: "module" })
    );

    this.#workers.forEach(worker => {
      const { promise, resolve } = Promise.withResolvers();
      worker.ready = promise;
      worker.resolveReadyPromise = resolve;
      worker.hasPendingRequests = false;
      worker.onmessage = event => this.#handleWorkerMessage(event, worker);
    });

    this.#broadcastWasmBinary();
  }

  /**
   * Sends a solutions request to the worker pool for a given `requestId` and input text.
   *
   * @param {number} requestId - The unique ID assigned to this request.
   * @param {string} input - The puzzle text or letters from which to find valid solutions.
   * @returns {Promise<void>}
   */
  sendSolutionsRequest(requestId, input) {
    this.#activeRequestId = requestId;

    // Reset any previously stored solutions
    this.#resetSolutions();
    this.#activeWorkerCount = this.#workers.length;

    /** @type {ValidWordsRequest} */
    const validWordsRequest = {
      type: "ValidWordsRequest",
      requestId,
      text: input,
    };

    const worker = this.#workers[requestId % this.#workers.length];

    worker.ready.then(() => {
      if (requestId !== this.#activeRequestId) {
        // This request is no longer relevant.
        return;
      }

      worker.postMessage(validWordsRequest);
    });
  }

  /**
   * Clears the internal arrays that store solutions across different word counts.
   * This is useful for preparing the solver pool for a fresh request.
   */
  #resetSolutions() {
    this.#solutions = [
      [], // 1-word solutions
      [], // 2-word solutions
      [], // 3-word solutions
      [], // 4-word solutions
      [], // 5-word solutions
    ];
  }

  /**
   * Fetches the WASM binary, then broadcasts an "InitializeWasmRequest" message to each worker.
   * @returns {Promise<void>}
   */
  async #broadcastWasmBinary() {
    try {
      const response = await fetch(WASM_BINARY_PATH);
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM: ${response.statusText}`);
      }

      const wasmBinary = await response.arrayBuffer();

      // Broadcast the binary to each worker
      this.#workers.forEach(worker => {
        /** @type {InitializeWasmRequest} */
        const initRequest = {
          type: "InitializeWasmRequest",
          wasmBinary,
        };
        worker.postMessage(initRequest);
      });
    } catch (error) {
      console.error("Failed to fetch or distribute WASM binary:", error);
    }
  }

  /**
   * Handles messages received from any worker thread.
   *
   * @param {MessageEvent<WorkerMessage>} event - The message event containing data from the worker.
   * @param {Worker} worker - The worker that sent this message.
   */
  #handleWorkerMessage(event, worker) {
    const { data } = event;
    const { type } = data;

    switch (type) {
      /**
       * Emitted by a worker when its WASM module initialization is complete.
       *
       * @type {InitializeWasmResponse}
       */
      case "WasmInitialized": {
        worker.resolveReadyPromise();
        break;
      }

      /**
       * Emitted by a worker in response to a "ValidWordsRequest" message,
       * indicating that the worker has loaded and serialized a list of valid words.
       *
       * The pool then divides this list among all workers to begin solution generation.
       *
       * @type {ValidWordsResponse}
       */
      case "ValidWordsResponse": {
        const { requestId } = data;
        if (requestId !== this.#activeRequestId) {
          // This request is no longer relevant.
          return;
        }

        const { serializedWords, wordCount } = data;
        const solverCount = this.#workers.length;

        // Divide the word list among all workers
        let rangeStart = 0;
        const sliceSize = Math.floor(wordCount / solverCount);
        const remainder = wordCount % solverCount;

        for (let workerIndex = 0; workerIndex < solverCount; workerIndex++) {
          const worker = this.#workers[workerIndex];
          worker.ready.then(() => {
            if (requestId !== this.#activeRequestId) {
            // This request is no longer relevant.
              return;
            }

            const additionalElement = workerIndex < remainder ? 1 : 0;
            const rangeEnd = rangeStart + sliceSize + additionalElement;
            worker.hasPendingRequests = true;

            /** @type {SolutionsRequest} */
            const solutionsRequest = {
              type: "SolutionsRequest",
              requestId,
              serializedWords,
              rangeStart,
              rangeEnd,
            };

            worker.postMessage(solutionsRequest);
            rangeStart = rangeEnd;
          });
        }

        break;
      }

      /**
       * Emitted by a worker when it has generated one or more solutions for its assigned range.
       * A single worker may emit multiple "SolutionsResponse" messages until it signals `isFinalResponse = true`.
       *
       * Each "SolutionsResponse" triggers a "SolutionsUpdated" custom event so the UI can update
       * partially (with intermediate results) or fully (with final results).
       *
       * @type {SolutionsResponse}
       */
      case "SolutionsResponse": {
        const {
          requestId,
          isFinalResponse,
          oneWordSolutions,
          twoWordSolutions,
          threeWordSolutions,
          fourWordSolutions,
          fiveWordSolutions,
        } = data;

        if (requestId !== this.#activeRequestId) {
          // This request is no longer relevant.
          return;
        }

        this.#solutions[0].push(...oneWordSolutions);
        this.#solutions[1].push(...twoWordSolutions);
        this.#solutions[2].push(...threeWordSolutions);
        this.#solutions[3].push(...fourWordSolutions);
        this.#solutions[4].push(...fiveWordSolutions);

        if (isFinalResponse) {
          worker.hasPendingRequests = false;
          this.#activeWorkerCount -= 1;
        }

        // Dispatch a "SolutionsUpdated" event so that the UI can refresh
        // `isFinalResponse` is true only if all workers are done.
        document.dispatchEvent(
          new CustomEvent("SolutionsUpdated", {
            detail: {
              requestId,
              solutions: this.#solutions,
              isFinalResponse: this.#activeWorkerCount == 0,
            },
          })
        );

        break;
      }

      default: {
        throw new Error(`Unknown message received from worker: ${type}`);
      }
    }
  }
}

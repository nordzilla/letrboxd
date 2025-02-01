/**
 * @typedef {import("./../types/message-data").SorterWorkerMessage} SorterWorkerMessage
 * @typedef {import("./../types/message-data").SortedSolutionsRequest} SortedSolutionsRequest
 * @typedef {import("./../types/message-data").SortedSolutionsResponse} SortedSolutionsResponse
 */

/**
 * The path to the CSS file for the SolutionList.
 */
const CSS_VARIABLES = "./site/styles/variables.css";

/**
 * The path to the CSS file for the SolutionList.
 */
const CSS_SOLUTION_LIST = "./site/styles/solution-list.css";

/**
 * The path to the sorter worker script used to sort asynchronously in a worker thread.
 * @type {string}
 */
const WORKER_PATH = "./site/workers/sorter.worker.mjs";

/**
 * A regular expression that captures a pair of letters separated by a space,
 * which might indicate a boundary between words.
 * @type {RegExp}
 */
const WORD_START_OR_END = /([A-Z] [A-Z])/g;

/**
 * A regular expression that captures the first letter of a string.
 * @type {RegExp}
 */
const FIRST_LETTER = /^([A-Z])/;

/**
 * A regular expression that captures the last letter of a string.
 * @type {RegExp}
 */
const LAST_LETTER = /([A-Z])$/;

/**
 * A custom web component that displays and manages a dynamically loaded,
 * scrollable, and keyboard-navigable list of solutions. It uses a dedicated
 * Web Worker to sort solutions asynchronously and supports chunk-based
 * incremental rendering as the user scrolls.
 */
export class SolutionList extends HTMLElement {
  /**
   * A stylesheet shared by all instances of this component.
   * @type {CSSStyleSheet}
   */
  static #STYLESHEET = new CSSStyleSheet();

  /**
   * The size of each chunk that is loaded during incremental rendering.
   * @type {number}
   */
  #CHUNK_SIZE = 500;

  /**
   * The main container element that holds the header and content.
   * @type {HTMLDivElement}
   */
  #container;

  /**
   * The header element that displays the component’s label, and which
   * can be clicked or keyed to expand/collapse the solution list.
   * @type {HTMLDivElement}
   */
  #header;

  /**
   * A label element within the header that provides accessible text
   * for the solution list header.
   * @type {HTMLLabelElement}
   */
  #headerLabel;

  /**
   * A span element inside the header label that displays the number of solutions.
   * @type {HTMLSpanElement}
   */
  #counterSpan;

  /**
   * The collapsible content area that holds the list of solutions.
   * @type {HTMLDivElement}
   */
  #content;

  /**
   * A container for the loading spinner and loading text.
   * @type {HTMLDivElement}
   */
  #loadingSpinner;

  /**
   * A span element that displays the current loading status (e.g., "Solving…" or "Sorting…").
   * @type {HTMLSpanElement}
   */
  #loadingText;

  /**
   * A list of solutions (words or phrases) to be displayed.
   * @type {string[]}
   */
  #solutions = [];

  /**
   * A DOMParser instance for converting raw HTML strings into DOM nodes
   * before inserting them into the component’s content area.
   * @type {DOMParser}
   */
  #parser = new DOMParser();

  /**
   * The ID associated with the current set of solutions, used to verify
   * that worker responses match the latest request.
   * @type {?number}
   */
  #activeRequestId = null;

  /**
   * A promise that resolves to the sorted solutions when the worker finishes.
   * @type {Promise<string[]> | null}
   */
  #sortedSolutionsPromise = null;

  /**
   * The function that resolves the #sortedSolutionsPromise.
   * @type {((sortedSolutions: string[]) => void) | null}
   */
  #resolveSortedSolutionsPromise = null;

  /**
   * The index of the currently focused solution item, used for keyboard navigation.
   * @type {number}
   */
  #focusedIndex = 0;

  /**
   * The current chunk index for incremental rendering; tracks how many
   * solutions have been appended so far.
   * @type {number}
   */
  #currentChunkIndex = 0;

  /**
   * A reference to the scroll event listener added during expansion, so
   * it can be removed on collapse or teardown.
   * @type {(event: Event) => void | null}
   */
  #scrollHandler = null;

  /**
   * A reference to the Web Worker instance used for off-main-thread sorting.
   * @type {Worker | null}
   */
  #worker = null;

  /**
   * An Intl.NumberFormat instance used to format large numbers.
   * @type {Intl.NumberFormat}
   */
  #numberFormatter = new Intl.NumberFormat();

  /**
   * A promise that resolves when the component has finished initializing (styles, DOM, etc.).
   * @type {Promise<void> | null}
   */
  #readyPromise = null;

  /**
   * The function that resolves the #readyPromise.
   * @type {(value?: void) => void}
   */
  #resolveReadyPromise = null;

  /**
   * Constructs a new SolutionList and sets up the shadow DOM.
   */
  constructor() {
    super();

    // Prepare a "ready" promise so external callers can await readiness if needed
    const { promise, resolve } = Promise.withResolvers();
    this.#readyPromise = promise;
    this.#resolveReadyPromise = resolve;

    // Attach Shadow DOM
    this.attachShadow({ mode: "open" });
  }

  /**
   * Called when the element is added to the DOM.
   */
  async connectedCallback() {
    this.style.visibility = "hidden";

    await Promise.all([
      this.#initStructure(),
      this.#initStyles(),
    ]);

    this.#attachListeners();
    this.#initializeWorker();
    this.#initializeHeaderText();

    this.#updateUI();
    this.#resolveReadyPromise();

    this.style.visibility = "visible";
  }

  /**
   * Called when the element is removed from the DOM.
   */
  disconnectedCallback() {
    this.#terminateWorker();
  }

  /**
   * Returns a promise that resolves when the component is fully ready (styles, DOM, etc.).
   * @returns {Promise<void>}
   */
  get ready() {
    return this.#readyPromise;
  }

  /**
   * Sets the current solutions to display.
   *
   * @param {string[]} solutions - An array of solutions to display.
   * @param {number} requestId - A unique identifier for the request.
   * @param {boolean} [isFinalResponse=true] - Indicates whether the provided solutions are final.
   */
  setSolutions(solutions, requestId, isFinalResponse = true) {
    if (requestId !== this.#activeRequestId) {
      // This is a new request, so destroy any previous state.
      this.#activeRequestId = requestId;
      this.#collapse();

      this.#sortedSolutionsPromise = null;
      this.#resolveSortedSolutionsPromise = null;
    }

    this.#solutions = solutions;

    this.#loadingText.textContent = isFinalResponse ? "Sorting…" : "Solving…";
    if (!this.#content.contains(this.#loadingSpinner)) {
      this.#content.innerHTML = "";
      this.#content.appendChild(this.#loadingSpinner);
    }

    if (isFinalResponse) {
      this.#requestSortedSolutions(requestId);
      if (this.#header.classList.contains("expanded")) {
        this.#expand(requestId);
      }
    }

    this.#updateUI(isFinalResponse);
  }

  /**
   * Moves focus to the most recently focused solution item.
   */
  focusMostRecentContentItem() {
    const items = Array.from(this.#content.querySelectorAll(".item-content"));
    if (items.length === 0) {
      return;
    }
    if (this.#focusedIndex < 0) {
      this.#focusedIndex = 0;
    } else if (this.#focusedIndex >= items.length) {
      this.#focusedIndex = items.length - 1;
    }
    items[this.#focusedIndex].focus();
  }

  /**
   * Fetches text content from a given URL.
   * @param {string} url - The URL to fetch.
   * @param {string} errorPrefix - A string to prepend to any error message.
   * @returns {Promise<string>}
   */
  async #fetchFileText(url, errorPrefix) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`${errorPrefix}: ${response.status} ${response.statusText}`);
      }
      return await response.text();
    } catch (err) {
      console.error(errorPrefix, err);
      throw err;
    }
  }

  /**
   * Dynamically fetches and applies external stylesheet content for this component.
   * @returns {Promise<void>}
   */
  async #initStyles() {
    const cssText = await Promise.all([
      this.#fetchFileText(CSS_VARIABLES),
      this.#fetchFileText(CSS_SOLUTION_LIST),
    ]).then(responseTexts => responseTexts.join("\n"));

    SolutionList.#STYLESHEET.replaceSync(cssText);
    this.shadowRoot.adoptedStyleSheets = [SolutionList.#STYLESHEET];
  }

  /**
   * Creates and appends the core DOM structure (container, header, content, spinner, etc.).
   */
  async #initStructure() {
    // Main container
    this.#container = document.createElement("div");
    this.#container.className = "solution-list-container";
    this.#container.setAttribute("translate", "no");

    // Header
    this.#header = document.createElement("div");
    this.#header.className = "solution-list-header";
    this.#header.setAttribute("role", "button");
    this.#header.setAttribute("aria-expanded", "false");

    // Label within header
    this.#headerLabel = document.createElement("label");
    this.#headerLabel.id = `solution-list-label-${Math.floor(Math.random() * 1e9)}`;
    this.#headerLabel.setAttribute("aria-hidden", "true");
    this.#header.setAttribute("aria-labelledby", this.#headerLabel.id);

    // Span for displaying the solution count
    this.#counterSpan = document.createElement("span");
    this.#counterSpan.setAttribute("translate", "no");

    // Build header label text
    this.#headerLabel.textContent = "";
    this.#headerLabel.appendChild(document.createTextNode(""));
    this.#headerLabel.appendChild(this.#counterSpan);
    this.#header.appendChild(this.#headerLabel);

    // Collapsible content area
    this.#content = document.createElement("div");
    this.#content.className = "solution-list-content";
    this.#content.style.display = "none";
    this.#content.tabIndex = -1;

    // Append header and content to main container
    this.#container.appendChild(this.#header);
    this.#container.appendChild(this.#content);

    // Create spinner and loading text
    const spinner = document.createElement("div");
    spinner.className = "spinner";
    this.#loadingText = document.createElement("span");
    this.#loadingText.textContent = "";
    this.#loadingSpinner = document.createElement("div");
    this.#loadingSpinner.className = "loading-indicator";
    this.#loadingSpinner.appendChild(spinner);
    this.#loadingSpinner.appendChild(this.#loadingText);

    // Append container to shadow DOM
    this.shadowRoot.appendChild(this.#container);
  }

  /**
   * Attaches various event listeners for the header and content.
   */
  #attachListeners() {
    this.#header.addEventListener("click", () => {
      if (this.#header.getAttribute("data-disabled") === "true") {
        return;
      }
      if (this.#header.classList.contains("expanded")) {
        this.#collapse();
        this.#header.blur();
      } else {
        this.#expand(this.#activeRequestId);
      }
    });

    this.#header.addEventListener("keydown", async (event) => {
      if (this.#header.getAttribute("data-disabled") === "true") {
        return;
      }
      switch (event.key) {
        case "Enter":
        case " ":
          event.preventDefault();
          if (this.#header.classList.contains("expanded")) {
            this.#collapse();
          } else {
            await this.#expand(this.#activeRequestId);
            this.#focusedIndex = 0;
            this.focusMostRecentContentItem();
          }
          break;
        case "ArrowDown":
        case "ArrowRight":
          event.preventDefault();
          this.focusNextContentOrHeader();
          break;
        case "ArrowUp":
        case "ArrowLeft":
          event.preventDefault();
          this.focusPreviousContentOrHeader();
          break;
      }
    });

    this.#content.addEventListener("focusin", (event) => {
      if (!event.target.classList.contains("item-content")) {
        return;
      }
      if (!event.target.hasAttribute("aria-label")) {
        this.#assignAriaLabelToItem(event.target);
      }
    });
  }

  /**
   * Initializes the Web Worker used for sorting solutions, setting up the
   * message handler and error handler.
   */
  #initializeWorker() {
    this.#worker = new Worker(WORKER_PATH, { type: "module" });
    this.#worker.onmessage = (event) => this.#handleWorkerMessage(event);
    this.#worker.onerror = (error) => {
      console.error("Worker encountered an error:", error);
    };
  }

  /**
   * Terminates the Web Worker, freeing resources.
   */
  #terminateWorker() {
    if (this.#worker) {
      this.#worker.terminate();
      this.#worker = null;
    }
  }

  /**
   * Handles incoming messages from the Web Worker. Specifically listens for
   * "SortedSolutionsResponse" to resolve the promise with the sorted solutions.
   *
   * @param {MessageEvent<SorterWorkerMessage>} event - The message from the worker.
   */
  #handleWorkerMessage(event) {
    const { data } = event;
    const { type } = data;

    switch (type) {
      case "SortedSolutionsResponse": {
        const { sortedSolutions, requestId } = data;
        if (requestId !== this.#activeRequestId) {
          // This request is no longer relevant.
          return;
        }
        this.#resolveSortedSolutionsPromise(sortedSolutions);
        break;
      }
      default:
        throw new Error(`Unknown message received from worker: ${type}`);
    }
  }

  /**
   * Updates the text displayed in the header label and counter span based on the
   * component’s "word-count" attribute and the current number of solutions.
   */
  #initializeHeaderText() {
    const wordCount = this.getAttribute("word-count") || "N";
    this.#headerLabel.textContent = "";
    this.#headerLabel.appendChild(
      document.createTextNode(`${wordCount}-Word Solutions: `)
    );
    this.#headerLabel.appendChild(this.#counterSpan);
  }

  /**
   * Updates the UI based on the current solutions. If there are no solutions
   * and we have received a final response, the header is disabled. Otherwise,
   * the header is enabled and displays the solution count.
   *
   * @param {boolean} [isFinalResponse=true] - Whether the data is final.
   */
  #updateUI(isFinalResponse = true) {
    this.#counterSpan.textContent = this.#numberFormatter.format(this.#solutions.length);

    if (this.#solutions.length === 0 && isFinalResponse) {
      this.#header.setAttribute("data-disabled", "true");
      this.#header.setAttribute("aria-hidden", "true");
      this.#header.tabIndex = -1;
      this.#collapse();
    } else if (this.#solutions.length > 0) {
      this.#header.removeAttribute("data-disabled");
      this.#header.removeAttribute("aria-hidden");
      this.#header.tabIndex = 0;
    }
  }

  /**
   * Expands the solution list content and appends the first chunk of sorted solutions.
   * Subsequent chunks are loaded on demand when the user scrolls.
   * 
   * @param {number} requestId - The id of the active request when expand was called.
   * @returns {Promise<void>} A promise that resolves after the first chunk is rendered.
   */
  async #expand(requestId) {
    const { promise, resolve, reject } = Promise.withResolvers();

    if (requestId !== this.#activeRequestId) {
      // This request is no longer relevant.
      reject();
      return promise;
    }

    if (
      this.#content.classList.contains("expanded") &&
      !this.#content.contains(this.#loadingSpinner)
    ) {
      // Already expanded with content, no further action needed.
      reject();
      return promise;
    }

    this.#header.setAttribute("aria-expanded", "true");
    this.#header.classList.add("expanded");
    this.#content.style.visibility = "visible";
    this.#content.style.display = "block";

    this.#currentChunkIndex = 0;
    this.#maybeRemoveScrollHandler();

    const sortedSolutions = await this.#sortedSolutionsPromise;
    if (requestId !== this.#activeRequestId) {
      // This request is no longer relevant.
      reject();
      return promise;
    }

    this.#content.innerHTML = "";
    this.#content.scrollTop = 0;

    this.#appendNextChunk(sortedSolutions, requestId, () => {
      resolve();
      this.#scrollHandler = this.#onScroll.bind(this, sortedSolutions, requestId);
      this.#content.addEventListener("scroll", this.#scrollHandler);
    });

    return promise;
  }

  /**
   * Collapses the solution list by hiding the content, clearing it, and removing
   * the scroll event listener and expanded state.
   */
  #collapse() {
    this.#maybeRemoveScrollHandler();
    this.#header.classList.remove("expanded");
    this.#header.setAttribute("aria-expanded", "false");
    this.#content.style.visibility = "hidden";
    this.#content.style.display = "none";
    this.#content.innerHTML = "";
    this.#focusedIndex = 0;
  }

  /**
   * Sends a "SortedSolutionsRequest" message to the worker to sort the current solutions.
   *
   * @param {number} requestId - The ID to associate with this request.
   * @returns {Promise<string[]>} A promise that resolves with the sorted solutions.
   */
  #requestSortedSolutions(requestId) {
    const { promise, resolve } = Promise.withResolvers();
    this.#sortedSolutionsPromise = promise;
    this.#resolveSortedSolutionsPromise = resolve;

    /** @type {SortedSolutionsRequest} */
    const request = {
      type: "SortedSolutionsRequest",
      requestId,
      solutions: this.#solutions,
    };

    requestIdleCallback(() => {
      if (requestId !== this.#activeRequestId) {
        // This request is no longer relevant.
        return;
      }
      this.#worker.postMessage(request);
    });

    return promise;
  }

  /**
   * Appends the next chunk of sorted solutions to the content area, starting
   * at #currentChunkIndex. Once done, calls the optional callback.
   *
   * @param {string[]} sortedSolutions - The already-sorted solutions.
   * @param {number} requestId - The current request identifier.
   * @param {() => void} [doneCallback] - A callback invoked after the chunk is appended.
   */
  #appendNextChunk(sortedSolutions, requestId, doneCallback) {
    if (requestId !== this.#activeRequestId) {
      if (typeof doneCallback === "function") {
        doneCallback();
      }
      return;
    }

    const start = this.#currentChunkIndex;
    const end = Math.min(start + this.#CHUNK_SIZE, sortedSolutions.length);
    if (start >= sortedSolutions.length) {
      if (typeof doneCallback === "function") {
        doneCallback();
      }
      return;
    }

    const digitCount = String(sortedSolutions.length).length;
    let chunkHTML = "";

    for (let i = start; i < end; i++) {
      chunkHTML += this.#createSolutionListItemHTML(sortedSolutions[i], i, digitCount);
    }

    const doc = this.#parser.parseFromString(chunkHTML, "text/html");
    const existingItemCount = this.#container.querySelectorAll(".solution-list-item").length;
    const newItems = doc.body.querySelectorAll(".solution-list-item");
    this.#content.append(doc.body);

    newItems.forEach((itemElement, index) => {
      index += existingItemCount;
      const contentElement = itemElement.querySelector(".item-content");
      contentElement.tabIndex = -1;
      contentElement.addEventListener("focus", () => this.#moveFocus(index));
      contentElement.addEventListener("keydown", (e) => this.#onSolutionKeyDown(e, contentElement));
    });

    this.#currentChunkIndex = end;
    setTimeout(() => {
      if (typeof doneCallback === "function") {
        doneCallback();
      }
    }, 0);
  }

  /**
   * Handles the scroll event on the content area to load additional chunks of solutions
   * when the user nears the bottom of the scroll area.
   *
   * @param {string[]} sortedSolutions - The sorted solutions to display.
   * @param {number} requestId - The ID for verifying the request is still valid.
   */
  #onScroll(sortedSolutions, requestId) {
    if (requestId !== this.#activeRequestId) {
      this.#maybeRemoveScrollHandler();
      return;
    }

    if (!this.#header.classList.contains("expanded")) {
      this.#maybeRemoveScrollHandler();
      return;
    }

    const nearBottom =
      this.#content.scrollTop + this.#content.clientHeight >=
      this.#content.scrollHeight * 0.8;

    if (nearBottom) {
      this.#appendNextChunk(sortedSolutions, requestId);
    }
  }

  /**
   * Removes the scroll event listener from the content area.
   */
  #maybeRemoveScrollHandler() {
    if (this.#scrollHandler) {
      this.#content.removeEventListener("scroll", this.#scrollHandler);
      this.#scrollHandler = null;
    }
  }

  /**
   * Creates an HTML string for a single solution-list item. This includes
   * formatting certain letters as bold and adding a padded item number.
   *
   * @param {string} item - The solution text.
   * @param {number} index - The index of the solution in the sorted list.
   * @param {number} digitCount - The total number of digits to display for item numbers.
   * @returns {string} The HTML markup for the item.
   */
  #createSolutionListItemHTML(item, index, digitCount) {
    const number = String(index + 1).padStart(digitCount, "0");
    const unpaddedPart = String(index + 1);
    const paddingLength = digitCount - unpaddedPart.length;
    const paddedPart = number.slice(0, paddingLength);
    const nonPaddedPart = number.slice(paddingLength);

    const formattedText = item
      .replace(WORD_START_OR_END, "<span class=\"highlight-bold\">$1</span>")
      .replace(FIRST_LETTER, "<span class=\"highlight-bold\">$1</span>")
      .replace(LAST_LETTER, "<span class=\"highlight-bold\">$1</span>");

    return `
      <div class="solution-list-item" role="row">
        <span class="item-number">
          ${paddedPart}<span class="highlight-bold">${nonPaddedPart}</span>
        </span>
        <span class="item-content" role="cell">${formattedText}</span>
      </div>
    `;
  }

  /**
   * Handles keyboard events on individual solution items. Navigates between
   * items or between solution-list headers according to arrow keys and Tab/Shift+Tab.
   *
   * @param {KeyboardEvent} event - The keyboard event object.
   * @param {HTMLElement} currentItem - The currently focused solution item.
   */
  #onSolutionKeyDown(event, currentItem) {
    const items = Array.from(this.#content.querySelectorAll(".item-content"));
    const currentIndex = items.indexOf(currentItem);

    switch (event.key) {
      case "ArrowDown":
      case "ArrowRight": {
        event.preventDefault();
        const nextIndex = currentIndex + 1;
        if (nextIndex < items.length) {
          this.#moveFocus(nextIndex);
        } else {
          this.focusNextHeader();
        }
        break;
      }
      case "ArrowUp":
      case "ArrowLeft": {
        event.preventDefault();
        const prevIndex = currentIndex - 1;
        if (prevIndex >= 0) {
          this.#moveFocus(prevIndex);
        } else {
          this.#header.focus();
        }
        break;
      }
      case "Tab": {
        event.preventDefault();
        if (event.shiftKey) {
          this.#header.focus();
        } else {
          this.focusNextHeader();
        }
        break;
      }
    }
  }

  /**
   * Moves focus from the currently focused item to the item at targetIndex,
   * updating tabIndex as needed.
   *
   * @param {number} targetIndex - The index of the solution item to focus.
   */
  #moveFocus(targetIndex) {
    const items = Array.from(this.#content.querySelectorAll(".item-content"));
    items[this.#focusedIndex].tabIndex = -1;
    this.#focusedIndex = targetIndex;
    items[targetIndex].tabIndex = 0;
    items[targetIndex].focus();
  }

  /**
   * Focuses on the previous solution list’s header or its last item if it’s expanded.
   * If the previous header is disabled, recursively attempts the next earlier list.
   */
  focusPreviousContentOrHeader() {
    const allLists = Array.from(document.querySelectorAll("solution-list"));
    const currentIndex = allLists.indexOf(this);
    if (currentIndex === 0) {
      return;
    }

    const previousSolutionList = allLists[currentIndex - 1];
    const previousHeader = previousSolutionList.shadowRoot.querySelector(".solution-list-header");
    if (previousHeader.getAttribute("data-disabled") === "true") {
      return previousSolutionList.focusPreviousContentOrHeader();
    }
    if (previousHeader.classList.contains("expanded")) {
      return previousSolutionList.focusMostRecentContentItem();
    }
    previousHeader.focus();
  }

  /**
   * Focuses on the next solution list’s content item if expanded or its header otherwise.
   */
  focusNextContentOrHeader() {
    if (this.#header.classList.contains("expanded")) {
      this.focusMostRecentContentItem();
      return;
    }
    this.focusNextHeader();
  }

  /**
   * Focuses on the next solution list’s header or, if disabled, continues onward.
   */
  focusNextHeader() {
    const allLists = Array.from(document.querySelectorAll("solution-list"));
    const currentIndex = allLists.indexOf(this);
    if (currentIndex === -1 || currentIndex === allLists.length - 1) {
      return;
    }

    const nextSolutionList = allLists[currentIndex + 1];
    const nextHeader = nextSolutionList.shadowRoot.querySelector(".solution-list-header");
    if (nextHeader.getAttribute("data-disabled") === "true") {
      return nextSolutionList.focusNextHeader();
    }
    nextHeader.focus();
  }

  /**
   * Assigns an ARIA label to the given solution item if it does not already have one.
   * This label includes the word count, the item’s numeric index, and spells
   * out the solution text for screen reader users.
   *
   * @param {HTMLElement} item - The DOM element representing the solution content.
   */
  #assignAriaLabelToItem(item) {
    const wordCount = this.getAttribute("word-count") || "N";
    const solutionListItem = item.closest(".solution-list-item");
    const numberEl = solutionListItem?.querySelector(".item-number");
    const numberString = (numberEl?.textContent ?? "").replace(/\D+/g, "");
    const numericIndex = parseInt(numberString, 10) || 1;
    const rawText = item.textContent || "";
    const spelledOut = rawText.split("").join(" ").replaceAll("  ", ": ");
    item.setAttribute(
      "aria-label",
      `${wordCount}-Word Solution number ${numericIndex}, ${spelledOut}`
    );
  }
}

customElements.define("solution-list", SolutionList);

import { A11yAnnouncer } from "./a11y-announcer.mjs";

/**
 * Determines if the current user agent is Safari.
 * @returns {boolean}
 */
function isSafari() {
  return (
    navigator.userAgent.includes("Safari") &&
    !navigator.userAgent.includes("Chrome") &&
    !navigator.userAgent.includes("CriOS")
  );
}

/**
 * The path to the SVG file for the LetrBoxd input.
 * @type {string}
 */
const SVG_PATH = "./site/svg/letrboxd-input.svg";

/**
 * The path to the CSS file for the LetrBoxd input.
 * @type {string}
 */
const CSS_VARIABLES = "./site/styles/variables.css";

/**
 * The path to the CSS file for the LetrBoxd input.
 * @type {string}
 */
const CSS_LETRBOXD_INPUT = "./site/styles/letrboxd-input.css";

/**
 * A regular expression that matches any ASCII alphabetic character (uppercase or lowercase).
 *
 * @type {RegExp}
 */
const ASCII_ALPHABETIC = /[A-Za-z]/;

/**
 * A custom web component for displaying a LetrBoxd input interface with accessibility enhancements.
 *
 * This component provides an SVG-based input interface with four sides, each containing three input boxes.
 * It allows users to enter letters, validates the input, and manages focus navigation between elements.
 * It also uses a dedicated A11yAnnouncer class for ARIA labels and live region announcements, and
 * announces puzzle status (blanks and duplicates) after each letter input.
 */
export class LetrBoxdInput extends HTMLElement {
  /**
   * The total number of input elements in the puzzle.
   * @type {number}
   */
  static #INPUT_COUNT = 12;

  /**
   * The default fill color for blank or inactive elements.
   * @type {string}
   */
  static #DEFAULT_FILL_COLOR = "#F8F8F8";

  /**
   * A shared stylesheet that will be adopted by this component.
   * @type {CSSStyleSheet}
   */
  static #STYLESHEET = new CSSStyleSheet();

  /**
   * A descriptive label for each input position, used for ARIA announcements.
   * @type {string[]}
   */
  static #POSITIONS = [
    "top-side left value: ", "top-side center value: ", "top-side right value: ",
    "right-side upper value: ", "right-side middle value: ", "right-side bottom value: ",
    "bottom-side left value: ", "bottom-side center value: ", "bottom-side right value: ",
    "left-side top value: ", "left-side middle value: ", "left-side bottom value: ",
  ];

  /**
   * A list of the text input elements contained in the SVG.
   * @type {HTMLElement[]}
   */
  #inputElements = [];

  /**
   * A list of the circle elements contained in the SVG.
   * @type {SVGCircleElement[]}
   */
  #circleElements = [];

  /**
   * A list of the rect elements contained in the SVG.
   * @type {SVGRectElement[]}
   */
  #rectElements = [];

  /**
   * The index of the currently focused input element, or null if none are focused.
   * @type {?number}
   */
  #focusedIndex = null;

  /**
   * The letters displayed on the top side.
   * @type {string}
   */
  #top = "___";

  /**
   * The letters displayed on the bottom side.
   * @type {string}
   */
  #bottom = "___";

  /**
   * The letters displayed on the right side.
   * @type {string}
   */
  #right = "___";

  /**
   * The letters displayed on the left side.
   * @type {string}
   */
  #left = "___";

  /**
   * The previously reported value for dispatching "InputChanged" events.
   * @type {string}
   */
  #previousValue = "";

  /**
   * A date (YYYY-MM-DD) to display in the SVG.
   * @type {?string}
   */
  #date = null;

  /**
   * A live region announcer for assistive technology.
   * @type {A11yAnnouncer}
   */
  #announcer = new A11yAnnouncer();

  /**
   * A promise that resolves once the SVG and CSS are fully loaded.
   * @type {Promise<void> | null}
   */
  #readyPromise = null;

  /**
   * The function that resolves the #readyPromise.
   * @type {(value?: void) => void}
   */
  #resolveReadyPromise = null;

  /**
   * Constructs a new LetrBoxdInput and sets up the shadow DOM.
   */
  constructor() {
    super();

    this.attachShadow({ mode: "open" });

    const { promise, resolve } = Promise.withResolvers();
    this.#readyPromise = promise;
    this.#resolveReadyPromise = resolve;
  }

  /**
   * Called when the element is added to the DOM.
   */
  async connectedCallback() {
    this.style.visibility = "hidden";

    await Promise.all([
      this.#initSVG(),
      this.#initStyles(),
    ]);

    this.#updateVisualStates();
    this.#updateDateText();
    this.#resolveReadyPromise();

    this.style.visibility = "visible";
  }

  /**
   * Called when the element is removed from the DOM.
   */
  disconnectedCallback() {
    this.#announcer.disconnect();
  }

  /**
   * Gets or sets the letters on the top side.
   * @type {string}
   */
  get top() {
    return this.#top;
  }
  set top(value) {
    if (this.#setSideValueByIndex(0, value)) {
      this.#updateInputElementsFromValue();
      this.#maybeDispatchInputChangedEvent();
    }
  }

  /**
   * Gets or sets the letters on the right side.
   * @type {string}
   */
  get right() {
    return this.#right;
  }
  set right(value) {
    if (this.#setSideValueByIndex(3, value)) {
      this.#updateInputElementsFromValue();
      this.#maybeDispatchInputChangedEvent();
    }
  }

  /**
   * Gets or sets the letters on the bottom side.
   * @type {string}
   */
  get bottom() {
    return this.#bottom;
  }
  set bottom(value) {
    if (this.#setSideValueByIndex(6, value)) {
      this.#updateInputElementsFromValue();
      this.#maybeDispatchInputChangedEvent();
    }
  }

  /**
   * Gets or sets the letters on the left side.
   * @type {string}
   */
  get left() {
    return this.#left;
  }
  set left(value) {
    if (this.#setSideValueByIndex(9, value)) {
      this.#updateInputElementsFromValue();
      this.#maybeDispatchInputChangedEvent();
    }
  }

  /**
   * Gets or sets the combined value of all 12 inputs.
   * @type {string}
   */
  get value() {
    return `${this.top}${this.right}${this.bottom}${this.left}`;
  }
  set value(value) {
    const oldValue = this.value;
    const sanitizedValue = LetrBoxdInput.#sanitizeLetters(value, 12);
    if (oldValue === sanitizedValue) {
      return;
    }
    const [top, right, bottom, left] = sanitizedValue.match(/.{3}/g);
    this.#top = top;
    this.#right = right;
    this.#bottom = bottom;
    this.#left = left;
    this.#updateInputElementsFromValue();
    this.#maybeDispatchInputChangedEvent();
  }

  /**
   * Gets or sets the current puzzle date (YYYY-MM-DD). If invalid, clears it.
   * @type {?string}
   */
  get date() {
    return this.#date;
  }
  set date(value) {
    this.#date = value;
    this.#updateDateText();
  }

  /**
   * Returns a promise that resolves when the component is fully ready.
   * @returns {Promise<void>}
   */
  get ready() {
    return this.#readyPromise;
  }

  /**
   * Removes focus from all inputs, clearing the focused index.
   */
  clearFocus() {
    this.#focusedIndex = null;
    this.#updateVisualStates();
  }

  /**
   * Clears the entire puzzle, removing any date and resetting inputs.
   */
  clear() {
    this.date = null;
    this.value = "____________";
    this.clearFocus();
  }

  /**
   * Determines if the current puzzle input is valid (no underscores, no duplicates).
   * @returns {boolean} True if valid; otherwise, false.
   */
  isInputValid() {
    const input = this.value;
    if (!/^[A-Z]{12}$/.test(input)) {
      return false;
    }
    const uniqueLetters = new Set(input);
    return uniqueLetters.size === 12;
  }

  /**
   * Generates all permutations of the puzzle sides when each side is sorted.
   * @returns {string[]} All permutations of the sorted side letters, ignoring side order.
   */
  normalizedValuePermutations() {
    const sortedTop = this.top.split("").sort().join("");
    const sortedRight = this.right.split("").sort().join("");
    const sortedBottom = this.bottom.split("").sort().join("");
    const sortedLeft = this.left.split("").sort().join("");
    const sortedSides = [sortedTop, sortedRight, sortedBottom, sortedLeft];

    function getPermutations(arr) {
      if (arr.length <= 1) {
        return [arr];
      }
      const perms = [];
      for (let index = 0; index < arr.length; index++) {
        const current = arr[index];
        const remaining = [...arr.slice(0, index), ...arr.slice(index + 1)];
        for (const perm of getPermutations(remaining)) {
          perms.push([current, ...perm]);
        }
      }
      return perms;
    }

    return getPermutations(sortedSides).map(perm => perm.join(""));
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
   * Loads the external SVG file and inserts it into the shadow DOM,
   * Registers references and event listeners for the SVG's components.
   */
  async #initSVG() {
    try {
      const svgText = await this.#fetchFileText(
        SVG_PATH,
        "Error loading LetrBoxdInput SVG"
      );
      const parser = new DOMParser();
      const svgDoc = parser.parseFromString(svgText, "image/svg+xml");
      const svgElem = svgDoc.querySelector("svg");

      this.shadowRoot.textContent = "";
      this.shadowRoot.appendChild(svgElem);

      for (let index = 0; index < LetrBoxdInput.#INPUT_COUNT; index++) {
        const circle = this.shadowRoot.getElementById(`circle${index}`);
        const rect = this.shadowRoot.getElementById(`rect${index}`);
        const input = this.shadowRoot.getElementById(`input${index}`);

        if (isSafari()) {
          this.#removeClassFromElements([circle, rect, input], "growable");
        }

        if (circle) {
          this.#circleElements.push(circle);
        }
        if (rect) {
          this.#rectElements.push(rect);
        }
        if (input) {
          input.textContent = "_";
          this.#inputElements.push(input);
        }
      }
      this.#attachListeners();
    } catch (e) {
      console.error("Error loading LetrBoxdInput SVG:", e);
      this.shadowRoot.textContent = "Error loading input interface.";
    }
  }

  /**
   * Dynamically fetches and applies external stylesheet content for this component.
   * @returns {Promise<void>}
   */
  async #initStyles() {
    const cssText = await Promise.all([
      this.#fetchFileText(CSS_VARIABLES),
      this.#fetchFileText(CSS_LETRBOXD_INPUT),
    ]).then(responseTexts => responseTexts.join("\n"));

    LetrBoxdInput.#STYLESHEET.replaceSync(cssText);
    this.shadowRoot.adoptedStyleSheets = [LetrBoxdInput.#STYLESHEET];
  }

  /**
   * Removes a given class from an array of elements, if they exist.
   * @param {(HTMLElement|SVGElement)[]} elements - Elements from which to remove the class.
   * @param {string} className - The class name to remove.
   */
  #removeClassFromElements(elements, className) {
    elements.forEach(el => {
      if (el) {
        el.classList.remove(className);
      }
    });
  }

  /**
   * Sets up all event listeners on circles, rects, and inputs.
   */
  #attachListeners() {
    for (let index = 0; index < LetrBoxdInput.#INPUT_COUNT; index++) {
      const circleElement = this.#circleElements[index];
      const rectElement = this.#rectElements[index];
      const inputElement = this.#inputElements[index];

      this.#attachInputListeners(index, inputElement);
      this.#attachClickListeners(index, [circleElement, rectElement, inputElement]);
      this.#attachHoverListeners([circleElement, rectElement, inputElement]);
    }
  }

  /**
   * Attaches click listeners to elements so they highlight together.
   * @param {number} index - The index of the element on which to attach click listeners.
   * @param {(HTMLElement|SVGElement)[]} elements - The elements to attach click listeners to.
   */
  #attachClickListeners(index, elements) {
    elements.forEach(element => {
      element.addEventListener("click", () => {
        this.#focusIndex(index);
      });
    });
  }

  /**
   * Attaches hover listeners to elements so they highlight together.
   * @param {(HTMLElement|SVGElement)[]} elements - The elements to attach hover listeners to.
   */
  #attachHoverListeners(elements) {
    elements.forEach(element => {
      element.addEventListener("mouseenter", () => {
        elements.forEach(el => el.classList.add("hovered"));
      });
      element.addEventListener("mouseleave", () => {
        elements.forEach(el => el.classList.remove("hovered"));
      });
    });
  }

  /**
   * Attaches keyboard and focus listeners to an input element.
   * @param {number} index - The index of the input element.
   * @param {HTMLElement} input - The input element.
   */
  #attachInputListeners(index, input) {
    if (!input) {
      return;
    }
    input.addEventListener("keydown", event => {
      if (event.ctrlKey || event.metaKey) {
        return;
      }
      switch (event.key) {
        case "Escape":
          input.blur();
          event.preventDefault();
          break;
        case "Backspace":
        case "Delete":
          this.#removeLetter(index);
          event.preventDefault();
          break;
        case "ArrowRight":
        case "ArrowDown":
          this.#focusIndex(this.#nextFocusIndex(index));
          event.preventDefault();
          break;
        case "ArrowLeft":
        case "ArrowUp":
          this.#focusIndex(this.#previousFocusIndex(index));
          event.preventDefault();
          break;
        default:
          if (event.key.length === 1 && ASCII_ALPHABETIC.test(event.key)) {
            this.#handleLetterInput(event, index);
          } else if (event.key !== "Tab") {
            event.preventDefault();
          }
      }
    });
    input.addEventListener("focus", () => {
      this.#focusIndex(index);
      this.#updateAriaStatus({ includePuzzleStatus: true });
    });
    input.addEventListener("blur", () => {
      this.#focusedIndex = null;
      this.#updateVisualStates();
    });
    input.addEventListener("click", () => {
      this.#focusIndex(index);
    });
  }

  /**
   * Handles a single letter input from the user.
   * @param {KeyboardEvent} event - The keyboard event.
   * @param {number} index - The index of the input receiving the letter.
   */
  #handleLetterInput(event, index) {
    event.preventDefault();

    const letter = event.key.toUpperCase();
    this.#updateLetterAtIndex(index, letter);

    const replacedIndex = this.#resetDuplicateLetters(letter, index);
    const nextIndex = replacedIndex ?? index;

    this.#updateVisualStates();

    const inputAnnouncement = `Entered letter: ${letter}, in the ${LetrBoxdInput.#POSITIONS[index]} `;

    let focusAnnouncement = "";
    if (replacedIndex && this.#inputElements[nextIndex].textContent === "_") {
      focusAnnouncement += `This replaced the preexisting letter: ${letter}, in the ${LetrBoxdInput.#POSITIONS[nextIndex]} `;
      focusAnnouncement += `Focus is now on: ${this.#inputElements[nextIndex].textContent}, in the ${LetrBoxdInput.#POSITIONS[nextIndex]} `;
    }

    this.#focusIndex(nextIndex);
    this.#maybeDispatchInputChangedEvent();

    this.#updateAriaStatus({
      message: `${inputAnnouncement} ${focusAnnouncement}`,
      includePuzzleStatus: true,
    });
  }

  /**
   * Removes a letter at a given index, replacing it with an underscore.
   * @param {number} index - The index of the letter to remove.
   */
  #removeLetter(index) {
    const removalAnnouncement = `Removed letter from ${LetrBoxdInput.#POSITIONS[index]} `;
    this.#updateLetterAtIndex(index, "_");
    this.#updateVisualStates();
    this.#updateAriaStatus({
      message: removalAnnouncement,
      includePuzzleStatus: true,
    });
    this.#maybeDispatchInputChangedEvent();
  }

  /**
   * Updates the letter at the specified index, both in the input element and the side strings.
   * @param {number} index - The input index.
   * @param {string} letter - The letter to set.
   */
  #updateLetterAtIndex(index, letter) {
    this.#setInputValue(index, letter);
    const sideValue = this.#getSideValueByIndex(index);
    const chars = sideValue.split("");
    chars[index % 3] = letter;
    this.#setSideValueByIndex(index, chars.join(""));
  }

  /**
   * Updates the date text in the SVG or clears it if invalid or null.
   */
  #updateDateText() {
    const dateText = this.shadowRoot.getElementById("dateText");
    if (!dateText) {
      return;
    }
    if (!this.isInputValid()) {
      this.#date = null;
    }
    if (!this.#date) {
      dateText.textContent = "";
      return;
    }
    const [year, month, day] = this.#date.split("-").map(Number);
    const maybeDate = new Date(year, month - 1, day);
    if (isNaN(maybeDate.getTime())) {
      dateText.textContent = "";
      return;
    }
    if (this.#isToday(maybeDate)) {
      dateText.textContent = "Today's Puzzle";
    } else {
      const monthName = maybeDate.toLocaleString("default", { month: "long" });
      const dayNum = maybeDate.getDate();
      const yearNum = maybeDate.getFullYear();
      dateText.textContent = `${monthName} ${dayNum}, ${yearNum}`;
    }
  }

  /**
   * Checks if a given date is today.
   * @param {Date} date - The date to check.
   * @returns {boolean} True if the provided date is the current date.
   */
  #isToday(date) {
    const today = new Date();
    return (
      date.getDate() === today.getDate() &&
      date.getMonth() === today.getMonth() &&
      date.getFullYear() === today.getFullYear()
    );
  }

  /**
   * Converts the given value to uppercase letters and underscores of a specific length.
   * @param {string} value - The value to sanitize.
   * @param {number} length - The desired length.
   * @returns {string} The sanitized value.
   */
  static #sanitizeLetters(value, length) {
    if (typeof value !== "string") {
      return "_".repeat(length);
    }
    return value
      .toUpperCase()
      .replace(/[^A-Z_]/g, "")
      .slice(0, length)
      .padEnd(length, "_");
  }

  /**
   * Returns the side value (top, right, bottom, left) for a given input index.
   * @param {number} index - The index.
   * @returns {string} The side value for that index.
   */
  #getSideValueByIndex(index) {
    if (index >= 0 && index <= 2) {
      return this.#top;
    } else if (index >= 3 && index <= 5) {
      return this.#right;
    } else if (index >= 6 && index <= 8) {
      return this.#bottom;
    }
    return this.#left;
  }

  /**
   * Sets the side value (top, right, bottom, left) for a given input index.
   * @param {number} index - The index.
   * @param {string} value - The new value for that side.
   * @returns {boolean} True if the side value actually changed; otherwise false.
   */
  #setSideValueByIndex(index, value) {
    const oldValue = this.#getSideValueByIndex(index);
    const sanitizedValue = LetrBoxdInput.#sanitizeLetters(value, 3);
    if (oldValue === sanitizedValue) {
      return false;
    }
    if (index >= 0 && index <= 2) {
      this.#top = value;
    } else if (index >= 3 && index <= 5) {
      this.#right = value;
    } else if (index >= 6 && index <= 8) {
      this.#bottom = value;
    } else {
      this.#left = value;
    }
    return true;
  }

  /**
   * Sets the value of the specified input element's textContent and ARIA label.
   * @param {number} index - The index of the element.
   * @param {string} value - The string to set.
   */
  #setInputValue(index, value) {
    this.#inputElements[index].textContent = value;
    this.#inputElements[index].setAttribute("aria-label", this.#getAriaLabelForIndex(index));
  }

  /**
   * Updates the displayed inputs based on the current side values.
   */
  #updateInputElementsFromValue() {
    const val = this.value;
    for (let index = 0; index < LetrBoxdInput.#INPUT_COUNT; index++) {
      this.#setInputValue(index, val[index]);
    }
    this.#updateVisualStates();
  }

  /**
   * Resets any duplicates of a newly inserted letter beyond the current index to underscores.
   * @param {string} letter - The newly inserted letter.
   * @param {number} currentIndex - The current index.
   * @returns {?number} The replaced index if a duplicate was found; otherwise null.
   */
  #resetDuplicateLetters(letter, currentIndex) {
    let replacedIndex = null;
    for (let index = 0; index < this.#inputElements.length; index++) {
      if (index !== currentIndex && this.#inputElements[index].textContent === letter) {
        replacedIndex = index;
        this.#updateLetterAtIndex(index, "_");
      }
    }
    return replacedIndex;
  }

  /**
   * Finds if there's a duplicate for the current letter and returns its index; otherwise -1.
   * @param {string} letter - The letter to check.
   * @param {number} currentIndex - The current index.
   * @returns {number} The index of the duplicate letter or -1 if none.
   */
  #findDuplicateIndex(letter, currentIndex) {
    if (letter === "_") {
      return -1;
    }
    for (let index = 0; index < this.#inputElements.length; index++) {
      if (index !== currentIndex && this.#inputElements[index].textContent === letter) {
        return index;
      }
    }
    return -1;
  }

  /**
   * Determines the next focus index in a custom clockwise pattern.
   * @param {number} currentIndex - The current focused index.
   * @returns {number} The next index to focus.
   */
  #nextFocusIndex(currentIndex) {
    const currentValue = this.#inputElements[currentIndex].textContent;
    const duplicateIndex = this.#findDuplicateIndex(currentValue, currentIndex);
    if (duplicateIndex !== -1) {
      return duplicateIndex;
    }
    switch (currentIndex) {
      case 5:  return 8;
      case 8:  return 7;
      case 7:  return 6;
      case 6:  return 11;
      case 11: return 10;
      case 10: return 9;
      case 9:  return 0;
      default: return currentIndex + 1;
    }
  }

  /**
   * Determines the previous focus index in a custom counterclockwise pattern.
   * @param {number} currentIndex - The current focused index.
   * @returns {number} The previous index to focus.
   */
  #previousFocusIndex(currentIndex) {
    const currentValue = this.#inputElements[currentIndex].textContent;
    const duplicateIndex = this.#findDuplicateIndex(currentValue, currentIndex);
    if (duplicateIndex !== -1) {
      return duplicateIndex;
    }
    switch (currentIndex) {
      case 8:  return 5;
      case 7:  return 8;
      case 6:  return 7;
      case 11: return 6;
      case 10: return 11;
      case 9:  return 10;
      case 0:  return 9;
      default: return currentIndex - 1;
    }
  }

  /**
   * Updates the fill colors and classes of all input, circle, and rect elements
   * to reflect focused state, blank letters, and duplicates.
   */
  #updateVisualStates() {
    const letterCounts = this.#getLetterCounts();
    this.#inputElements.forEach((_, index) => {
      this.#updateVisualStateForIndex(index, letterCounts);
    });
  }

  /**
   * Builds a count of the occurrences of each letter in the puzzle.
   * @returns {Object<string, number>} A mapping of letter -> count.
   */
  #getLetterCounts() {
    const letters = this.value;
    const letterCounts = {};
    for (const letter of letters) {
      letterCounts[letter] = (letterCounts[letter] || 0) + 1;
    }
    return letterCounts;
  }

  /**
   * Updates the visual state (colors, classes) for a given index.
   * @param {number} index - The index to update.
   * @param {Object<string, number>} letterCounts - A mapping of letters to their occurrence counts.
   */
  #updateVisualStateForIndex(index, letterCounts) {
    const circle = this.#circleElements[index];
    const rect = this.#rectElements[index];
    const input = this.#inputElements[index];
    if (!circle || !rect || !input) {
      return;
    }
    const inputValue = this.value[index];
    const isFocused = (index === this.#focusedIndex);
    [circle, rect, input].forEach(el => {
      el.classList.toggle("focused", isFocused);
    });
    let fillColor = LetrBoxdInput.#DEFAULT_FILL_COLOR;
    if (isFocused) {
      fillColor = "lightblue";
    } else if (inputValue === "_" || letterCounts[inputValue] > 1) {
      fillColor = "lightcoral";
    }
    rect.setAttribute("fill", fillColor);
    circle.setAttribute("fill", fillColor);
  }

  /**
   * Dispatches an "InputChanged" event if the value has changed since the last dispatch.
   */
  #maybeDispatchInputChangedEvent() {
    if (this.value === this.#previousValue) {
      return;
    }
    this.#date = null;
    this.#previousValue = this.value;
    document.dispatchEvent(new CustomEvent("InputChanged"));
    this.#updateDateText();
  }

  /**
   * Focuses the input at the given index and updates tabIndex.
   * @param {number} index - The index of the input to focus.
   */
  #focusIndex(index) {
    this.#focusedIndex = index;
    this.#updateTabIndex(index);
    this.#inputElements[index].focus();
    this.#updateVisualStates();
  }

  /**
   * Updates the tabIndex for all inputs, making only the focused index tabbable.
   * @param {number | null} focusIndex - The index of the focused input.
   */
  #updateTabIndex(focusIndex) {
    this.#inputElements.forEach((input, idx) => {
      if (idx === focusIndex) {
        input.tabIndex = 0;
        input.removeAttribute("aria-hidden");
      } else {
        input.tabIndex = -1;
        input.setAttribute("aria-hidden", "true");
      }
    });
  }

  /**
   * Builds the ARIA label for a given index.
   * @param {number} index - The index of the input element.
   * @returns {string} A descriptive label for screen readers.
   */
  #getAriaLabelForIndex(index) {
    const inputText = this.#inputElements[index].textContent;
    return LetrBoxdInput.#POSITIONS[index] + (inputText !== "_" ? inputText : "");
  }

  /**
   * Constructs a status message describing puzzle completion, blanks, and duplicates.
   * @returns {string} A descriptive puzzle status message.
   */
  #getPuzzleStatusMessage() {
    const letters = this.value;
    const underscores = [];
    const letterMap = new Map();
    for (let index = 0; index < letters.length; index++) {
      const char = letters[index];
      if (char === "_") {
        underscores.push(index);
      } else {
        if (!letterMap.has(char)) {
          letterMap.set(char, []);
        }
        letterMap.get(char).push(index);
      }
    }
    const duplicates = [];
    for (const [letter, positions] of letterMap.entries()) {
      if (positions.length > 1) {
        duplicates.push(letter);
      }
    }
    if (underscores.length === 0 && duplicates.length === 0) {
      let message;
      if (this.#date) {
        message = `Puzzle is complete with input from: ${this.#date}. `;
      } else {
        message = "Puzzle is complete! ";
      }
      message += `Top-side content: ${this.#top.split("").join(": ")}: `;
      message += `Right-side content: ${this.#right.split("").join(": ")}: `;
      message += `Bottom-side content: ${this.#bottom.split("").join(": ")}: `;
      message += `Left-side content: ${this.#left.split("").join(": ")}: `;
      return message.trim();
    }
    const blanksCount = underscores.length;
    const blanksPositions = underscores.map(i => this.#getAriaLabelForIndex(i)).join(", ");
    let message = `Puzzle is not complete. There are ${blanksCount} blanks remaining. `;
    if (blanksCount > 0) {
      message += `They are at: ${blanksPositions} `;
    }
    if (duplicates.length > 0) {
      message += `The following letters have duplicates: ${duplicates.join(", ")}. `;
    }
    return message.trim();
  }

  /**
   * Updates the hidden live region text for screen readers with optional puzzle status.
   * @param {Object} options - The status update options.
   * @param {string} [options.message=""] - A custom message to announce.
   * @param {boolean} [options.includePuzzleStatus=false] - Whether to include puzzle status.
   */
  #updateAriaStatus({
    message = "",
    includePuzzleStatus = false,
  }) {
    if (includePuzzleStatus) {
      message += this.#getPuzzleStatusMessage();
    }
    this.#announcer.announce(message);
  }
}

customElements.define("letrboxd-input", LetrBoxdInput);

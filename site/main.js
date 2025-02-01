import { LetrBoxdInput } from "./custom-elements/letrboxd-input.mjs";
import { SolutionList } from "./custom-elements/solution-list.mjs";
import { SolverPool } from "./solver-pool.mjs";

/**
 * @typedef {import("./types/event-data").SolutionsUpdatedDetail} SolutionsUpdatedDetail
 */

/**
 * A regex to match 3 characters at a time.
 */
const THREE_CHARACTERS = /.{3}/g;

/**
 * A promise that resolves to a mapping of dates to their corresponding input strings.
 * This data is fetched from a JSON file generated during the site's build process.
 * It is used to fetch a puzzle input from a given date.
 *
 * @type {Promise<Record<string, string>>}
 */
const inputsByDatePromise = fetch("./site/generated/json/inputsByDate.json")
  .then(response => response.json())
  .catch(error => console.error("Error loading inputsByDate JSON:", error));

/**
 * A promise that resolves to a mapping of puzzle inputs to their corresponding dates.
 * This data is fetched from a JSON file generated during the site's build process.
 * It is used to find the date associated with a given puzzle input.
 *
 * @type {Promise<Record<string, string>>}
 */
const datesByInputPromise = fetch("./site/generated/json/datesByInput.json")
  .then(response => response.json())
  .catch(error => console.error("Error loading datesByInput JSON:", error));

/**
 * A custom element to handle puzzle input in the UI.
 *
 * @type {LetrBoxdInput}
 */
const letrboxdInput = document.getElementById("input-board");

/**
 * Custom elements for displaying puzzle solutions in the UI.
 *
 * @type {NodeListOf<SolutionList>}
 */
const solutionLists = document.querySelectorAll("solution-list");

/**
 * A message displayed if a valid puzzle input has no found solutions.
 *
 * @type {HTMLElement}
 */
const noSolutionsCard = document.getElementById("no-solutions-card");

/**
 * A message displayed to inform the user of which word list the solvers use.
 *
 * @type {HTMLElement}
 */
const wordListCard = document.getElementById("word-list-card");

/**
 * A message displayed to describe the purpose of the site.
 *
 * @type {HTMLElement}
 */
const descriptionCard = document.getElementById("description-card");

/**
 * A message displayed to describe how to modify the puzzle input.
 *
 * @type {HTMLElement}
 */
const instructionsCard = document.getElementById("instructions-card");

/**
 * An element that displays the solution section header.
 *
 * @type {HTMLElement}
 */
const solutionsHeader = document.getElementById("solutions-header");

/**
 * A formatter to display solution counts.
 *
 * @type {Intl.NumberFormat}
 */
const numberFormat = new Intl.NumberFormat("en-US");

/**
 * Retrieves and validates the `solvers` parameter from the URL.
 * @returns {number | undefined} The number of solvers or undefined if invalid.
 */
function maybeGetSolverCountFromUrl() {
  const urlParams = new URLSearchParams(window.location.search);
  const solvers = parseInt(urlParams.get("solvers"), 10);

  if (isNaN(solvers) || solvers <= 0) {
    urlParams.delete("solvers");
    history.replaceState({}, "", `${window.location.pathname}?${urlParams.toString()}`);
    return undefined;
  }

  return solvers;
}

/**
 * The pool of solvers, which each run on their own worker thread.
 *
 * @type {SolverPool}
 */
const solverPool = new SolverPool(maybeGetSolverCountFromUrl());

/**
 * Prevents the next InputChanged event from pushing the URL state to the history.
 * This gets set whenever we pop a URL state from the history, to ensure that we
 * don't push the same state back on the stack twice.
 *
 * @type {boolean}
 */
let preventNextPushState = false;

/**
 * A monotonically increasing integer to track the request id that is relevant to the UI state.
 * This id is passed to the solver workers via messages and is returned in their responses.
 * If a response is received from a solver worker that contains a requestId that no longer
 * matches the activeRequestId, then the response is no longer relevant and should be ignored.
 *
 * @type {number}
 */
let activeRequestId = 0;

/**
 * Clears the solution lists for the current active request ID.
 * @param {number} requestId The ID of the current request.
 */
function clearSolutionLists(requestId) {
  noSolutionsCard.style.visibility = "hidden";
  for (const solutionList of solutionLists) {
    solutionList.setSolutions([], requestId);
  }
}

/**
 * Updates the URL state based on the input board and optional focused index.
 * Removes any extraneous URL parameters and pushes a new state unless prevented.
 */
function updateUrlState() {
  const newUrl = new URL(window.location.href);
  const { top, right, bottom, left, date } = letrboxdInput;

  const keysToRetain = date
    ? { date, solvers: maybeGetSolverCountFromUrl() }
    : { top, right, bottom, left, solvers: maybeGetSolverCountFromUrl() };

  const keysToRemove = [];
  newUrl.searchParams.forEach((_, key) => {
    if (!(key in keysToRetain)) {
      keysToRemove.push(key);
    }
  });

  keysToRemove.forEach(key => newUrl.searchParams.delete(key));

  Object.entries(keysToRetain).forEach(([key, value]) => {
    if (value !== undefined) {
      newUrl.searchParams.delete(key);
      newUrl.searchParams.set(key, value);
    }
  });

  if (preventNextPushState) {
    preventNextPushState = false;
  } else {
    window.history.pushState({}, "", newUrl);
  }
}

/**
 * Updates the LetrBoxdInput element's value from the current URL parameters.
 * If the URL contains a `date` parameter, that input is used.
 * Otherwise, if any top/right/bottom/left parameters are present, that grid-based input is used.
 * Otherwise, it falls back to the default puzzle input.
 */
function updateInputBoardFromURL() {
  letrboxdInput.clearFocus();

  const urlParams = new URLSearchParams(window.location.search);
  const dateParam = urlParams.get("date");
  const topParam = urlParams.get("top");
  const rightParam = urlParams.get("right");
  const bottomParam = urlParams.get("bottom");
  const leftParam = urlParams.get("left");

  if (dateParam) {
    inputsByDatePromise
      .then(inputs => {
        const input = inputs[dateParam];
        if (input) {
          letrboxdInput.value = input;
        } else {
          console.warn("Invalid or unknown date provided. Defaulting to today's puzzle.");
          setDefaultPuzzleInput();
        }
      })
      .catch(error => {
        console.warn("Failed to fetch inputs by date. Defaulting to a blank puzzle.", error);
        letrboxdInput.clear();
      });
  } else if (topParam || rightParam || bottomParam || leftParam) {
    letrboxdInput.value = `${topParam ?? "___"}${rightParam ?? "___"}${bottomParam ?? "___"}${leftParam ?? "___"}`;
  } else {
    setDefaultPuzzleInput();
  }
}

/**
 * Sets the puzzle input for today's date as defined in the inputsByDate JSON file.
 * If today's puzzle input is found, it updates the LetrBoxdInput element with that input.
 * If not, it logs a warning and defaults to a blank puzzle.
 */
function setDefaultPuzzleInput() {
  const date = new Date();
  const localDate = date.getFullYear()
    + "-" + String(date.getMonth() + 1).padStart(2, "0")
    + "-" + String(date.getDate()).padStart(2, "0");

  inputsByDatePromise
    .then(inputs => {
      const input = inputs[localDate];
      if (input) {
        const [top, right, bottom, left] = input.match(THREE_CHARACTERS);
        letrboxdInput.value = `${top}${right}${bottom}${left}`;
      } else {
        console.warn("Invalid or unknown date provided. Defaulting to a blank puzzle.");
        letrboxdInput.clear();
      }
    })
    .catch(error => {
      console.warn("Failed to fetch inputs by date. Defaulting to a blank puzzle.", error);
      letrboxdInput.clear();
    });
}

/**
 * Sets the displayed total solution count in the UI.
 * @param {number} count The total number of solutions found.
 */
function setSolutionsHeaderCount(count) {
  solutionsHeader.textContent = `Solutions: ${numberFormat.format(count)}`;
}

/**
 * Attempts to find a matching puzzle date for the current input by checking
 * all normalized permutations of the letters. If a match is found,
 * the `date` property of the input board is set to that date.
 *
 * @param {number} requestId - The ID of the current request to verify it's still valid.
 * @returns {Promise<void>}
 */
async function maybeMatchInputToDate(requestId) {
  const datesByInput = await datesByInputPromise;

  if (requestId !== activeRequestId) {
    return;
  }

  for (const value of letrboxdInput.normalizedValuePermutations()) {
    const matchingDate = datesByInput[value];
    if (matchingDate) {
      letrboxdInput.date = matchingDate;
      return;
    }
  }

  letrboxdInput.date = null;
}

/**
 * Handles the "InputChanged" custom event.
 * Increments the activeRequestId to invalidate older requests,
 * updates the URL state, hides the 'no solutions' message, and
 * requests solutions via the SolverPool if valid input is provided.
 */
document.addEventListener("InputChanged", async () => {
  const requestId = ++activeRequestId;

  setSolutionsHeaderCount(0);

  await maybeMatchInputToDate(requestId).then(updateUrlState);
  if (requestId !== activeRequestId) {
    return;
  }

  if (!letrboxdInput.isInputValid()) {
    clearSolutionLists(requestId);
    return;
  }

  solverPool.sendSolutionsRequest(requestId, letrboxdInput.value);
});

/**
 * Handles the "SolutionsUpdated" custom event.
 * Checks whether the request ID matches the active request.
 * Aggregates solution counts, updates each SolutionList, and
 * toggles the 'no solutions' message if necessary.
 */
document.addEventListener("SolutionsUpdated", (event) => {
  /** @type {SolutionsUpdatedDetail} */
  const { requestId, isFinalResponse, solutions } = event.detail;
  if (requestId !== activeRequestId) {
    return;
  }

  let totalSolutionCount = 0;
  solutionLists.forEach((solutionList, index) => {
    totalSolutionCount += solutions[index].length;
    solutionList.setSolutions(solutions[index], requestId, isFinalResponse);
  });

  setSolutionsHeaderCount(totalSolutionCount);
  if (totalSolutionCount || isFinalResponse) {
    noSolutionsCard.style.visibility = totalSolutionCount > 0 ? "hidden" : "visible";
  }
});

/**
 * Runs when the DOM is fully loaded. Sets up the initial URL-based puzzle input,
 * shows relevant messages, and adds a popstate listener to handle back/forward navigation.
 */
document.addEventListener("DOMContentLoaded", () => {
  window.addEventListener("popstate", () => {
    preventNextPushState = true;
    updateInputBoardFromURL();
  });

  const readyPromises = [
    letrboxdInput.ready,
    ...Array.from(solutionLists.values()).map(solutionList => solutionList.ready),
  ];


  Promise.all(readyPromises).then(() => {
    updateInputBoardFromURL();
    wordListCard.style.visibility = "visible";
    instructionsCard.style.visibility = "visible";
    descriptionCard.style.visibility = "visible";
  });
});

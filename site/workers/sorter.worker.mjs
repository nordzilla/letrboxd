/**
 * A Web Worker script to sort puzzle solutions without blocking the main thread.
 *
 * @typedef {import("./../types/message-data").SortedSolutionsRequest} SortedSolutionsRequest
 * @typedef {import("./../types/message-data").SortedSolutionsResponse} SortedSolutionsResponse
 */

/**
 * Sorts an array of solution strings based on the following rules:
 * 1. Each solution is split into an array of words using spaces as separators.
 * 2. Words are compared one by one in the following order of precedence:
 *    a. Words with fewer characters come first.
 *    b. If two words have the same length, they are sorted alphabetically.
 * 3. Comparison continues word by word until a difference is found.
 *
 * @param {string[]} solutions - An array of solution strings.
 * @returns {string[]} - A new array of solution strings sorted according to the specified rules.
 */
function sortSolutions(solutions) {
  const solutionsWithWords = solutions.map(solution => ({
    solution,
    words: solution.split(" "),
  }));

  solutionsWithWords.sort((lhs, rhs) => {
    const lhsWords = lhs.words;
    const rhsWords = rhs.words;

    // All solutions have the same count of words.
    const wordCount = lhsWords.length;

    for (let index = 0; index < wordCount; index++) {
      const lhsWord = lhsWords[index];
      const rhsWord = rhsWords[index];

      // Compare by word length
      const lengthDiff = lhsWord.length - rhsWord.length;
      if (lengthDiff !== 0) {
        return lengthDiff;
      }

      // Compare alphabetically if lengths are the same
      const alphabeticalDiff = lhsWord.localeCompare(rhsWord);
      if (alphabeticalDiff !== 0) {
        return alphabeticalDiff;
      }
    }

    console.error(
      "Found two equal solutions in the same solution list.",
      { lhsWords, rhsWords }
    );

    return 0;
  });

  // Extract the sorted original solution strings
  return solutionsWithWords.map(({ solution }) => solution);
}


/**
 * The main message handler for sorting operations.
 * This function listens for messages from the main thread, performs sorting on the provided solutions,
 * and sends the sorted results back.
 *
 * @param {MessageEvent<SortedSolutionsRequest>} event - The message event containing data from the main thread.
 */
self.onmessage = function (event) {
  const { solutions, requestId } = event.data;

  /** @type {SortedSolutionsResponse} */
  const response = {
    type: "SortedSolutionsResponse",
    requestId,
    sortedSolutions: sortSolutions(solutions),
  };

  self.postMessage(response);
};

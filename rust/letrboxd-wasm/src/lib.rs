//! This module provides WASM bindings for creating, managing, and retrieving valid words
//! represented by `LetterSequence` objects. It defines data structures and functions
//! for serializing, deserializing, and working with these letter sequences.

use letters::{create_letter_group_function, LetterSequence};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use word_list::WORDS;

thread_local! {
  /// Thread-local storage for the list of valid words. The list is registered by the worker
  /// thread and then used multiple times as it chunks the computation of finding solutions.
  static VALID_WORDS: RefCell<Vec<LetterSequence>> = RefCell::new(Vec::with_capacity(0));
}

/// A structure holding serialized words along with the total word count.
#[wasm_bindgen]
pub struct SerializedSequences {
  word_count: usize,
  serialized_words: Vec<u8>,
}

#[wasm_bindgen]
impl SerializedSequences {
  /// Returns the number of words in the serialized word list.
  #[must_use]
  #[wasm_bindgen(getter, js_name = "wordCount")]
  pub fn word_count(&self) -> usize {
    self.word_count
  }

  /// Returns the serialized list of valid words.
  #[must_use]
  #[wasm_bindgen(getter, js_name = "serializedWords")]
  pub fn serialized_words(&self) -> Vec<u8> {
    self.serialized_words.clone()
  }
}

/// A payload to hold solution strings grouped by how many words are in the solution.
/// There must be at least 1 word in a solution, and there can be at most 5 words.
#[wasm_bindgen]
#[derive(Default)]
pub struct SolutionsPayload {
  solutions_1_word: Vec<String>,
  solutions_2_word: Vec<String>,
  solutions_3_word: Vec<String>,
  solutions_4_word: Vec<String>,
  solutions_5_word: Vec<String>,
}

#[wasm_bindgen]
impl SolutionsPayload {
  /// Adds a [`LetterSequence`] solution to the relevant bucket based on the word count.
  pub fn push(&mut self, sequence: LetterSequence) {
    match sequence.word_count() {
      1 => self.solutions_1_word.push(sequence.solution_string()),
      2 => self.solutions_2_word.push(sequence.solution_string()),
      3 => self.solutions_3_word.push(sequence.solution_string()),
      4 => self.solutions_4_word.push(sequence.solution_string()),
      5 => self.solutions_5_word.push(sequence.solution_string()),
      n => unreachable!("Found a solution with {n} letters."),
    }
  }

  /// Takes and returns all one-word solutions, clearing them from the internal list.
  #[wasm_bindgen(getter, js_name = "oneWordSolutions")]
  pub fn take_one_word_solutions(&mut self) -> Vec<String> {
    std::mem::replace(&mut self.solutions_1_word, Vec::with_capacity(0))
  }

  /// Takes and returns all two-word solutions, clearing them from the internal list.
  #[wasm_bindgen(getter, js_name = "twoWordSolutions")]
  pub fn take_two_word_solutions(&mut self) -> Vec<String> {
    std::mem::replace(&mut self.solutions_2_word, Vec::with_capacity(0))
  }

  /// Takes and returns all three-word solutions, clearing them from the internal list.
  #[wasm_bindgen(getter, js_name = "threeWordSolutions")]
  pub fn take_three_word_solutions(&mut self) -> Vec<String> {
    std::mem::replace(&mut self.solutions_3_word, Vec::with_capacity(0))
  }

  /// Takes and returns all four-word solutions, clearing them from the internal list.
  #[wasm_bindgen(getter, js_name = "fourWordSolutions")]
  pub fn take_four_word_solutions(&mut self) -> Vec<String> {
    std::mem::replace(&mut self.solutions_4_word, Vec::with_capacity(0))
  }

  /// Takes and returns all five-word solutions, clearing them from the internal list.
  #[wasm_bindgen(getter, js_name = "fiveWordSolutions")]
  pub fn take_five_word_solutions(&mut self) -> Vec<String> {
    std::mem::replace(&mut self.solutions_5_word, Vec::with_capacity(0))
  }
}

/// Gathers valid words for a given 12-letter input, returning them in serialized form.
///
/// # Panics
///
/// Panics if the letter sequences cannot be serialized.
#[must_use]
#[wasm_bindgen(js_name = "getValidWords")]
pub fn get_valid_words(input: &str) -> SerializedSequences {
  let letter_group = create_letter_group_function!(input);

  let words = WORDS
    .iter()
    .copied()
    .filter(|word| word.is_valid_word(&letter_group))
    .collect::<Vec<_>>();

  SerializedSequences {
    word_count: words.len(),
    serialized_words: bincode::serialize(&words).unwrap(),
  }
}

/// Deserializes and stores valid words in thread-local storage for later use.
/// Solutions are generated in chunks, so this vector is reused multiple times.
///
/// # Panics
///
/// Panics if the serialized words cannot be deserialized.
#[wasm_bindgen(js_name = "registerValidWords")]
pub fn register_valid_words(serialized_words: &[u8]) {
  VALID_WORDS.replace(bincode::deserialize(serialized_words).unwrap());
}

/// Clears the currently registered valid words from thread-local storage.
#[wasm_bindgen(js_name = "clearValidWords")]
pub fn clear_valid_words() {
  VALID_WORDS.replace(Vec::with_capacity(0));
}

/// Generates puzzle solutions for valid words in the specified index range.
#[must_use]
#[wasm_bindgen]
pub fn solutions(range_start: usize, range_end: usize) -> SolutionsPayload {
  VALID_WORDS.with_borrow(|valid_words| {
    let mut solutions = SolutionsPayload::default();
    for index in range_start..range_end {
      solve_partition_once(valid_words[index], &mut solutions, valid_words);
    }

    solutions
  })
}

/// Recursively solves for valid 12-letter sequences, grouping the solutions by word count in the provided `SolutionsPayload`.
/// This version filters the valid words and then partitions them based on whether they are immediately appendable.
/// This strategy tends to be faster when the `valid_words` list is large, which is why we do it only for the first pass.
fn solve_partition_once(
  sequence: LetterSequence,
  solutions: &mut SolutionsPayload,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => {
      // If we have constructed a valid sequence with exactly 12 letters, it is a solution.
      solutions.push(sequence);
    }
    11 => {
      // There are no words that can be appended to an 11-letter sequence to form a 12-letter
      // solution because the minimum valid word length is 3 letters. This is a dead end.
    }
    _ => {
      let (appendable_words, remaining_valid_words) = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .partition::<Vec<_>, _>(|word| word.can_append_to(sequence));
      appendable_words.iter().copied().for_each(|word| {
        solve_filter(word.append_to(sequence), solutions, &remaining_valid_words);
      });
    }
  }
}

/// Recursively solves for valid 12-letter sequences, grouping the solutions by word count in the provided `SolutionsPayload`.
/// This version filters the valid words, but does not partition them based on their immediate appendability.
/// This strategy tends to be faster when the `valid_words` list is small.
fn solve_filter(
  sequence: LetterSequence,
  solutions: &mut SolutionsPayload,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => {
      // If we have constructed a valid sequence with exactly 12 letters, it is a solution.
      solutions.push(sequence);
    }
    11 => {
      // There are no words that can be appended to an 11-letter sequence to form a 12-letter
      // solution because the minimum valid word length is 3 letters. This is a dead end.
    }
    _ => {
      let remaining_valid_words = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .collect::<Vec<_>>();
      remaining_valid_words
        .iter()
        .copied()
        .filter(|word| word.can_append_to(sequence))
        .for_each(|word| {
          solve_filter(word.append_to(sequence), solutions, &remaining_valid_words);
        });
    }
  }
}

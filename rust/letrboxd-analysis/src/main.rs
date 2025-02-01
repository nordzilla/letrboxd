use itertools::Itertools;
use letters::{create_letter_group_function, LetterSequence, LetterSet};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
  str::{self},
  sync::RwLock,
};
use word_list::WORDS;

/// The set of vowels always included in the letter pool.
static VOWELS: &[u8] = b"AEIOU";

/// A subset of consonants you wish to include or exclude from your 7 chosen ones.
///
/// Uncomment or comment lines here to adjust which consonants are under consideration.
/// This example currently uses a small list of 7 consonants (`S, R, N, T, L, C, D`). 
/// For a larger set, uncomment more of these lines.
#[rustfmt::skip]
#[expect(clippy::byte_char_slices)]
static CONSONANTS: &[u8] = &[
    b'S',
    b'R',
    b'N',
    b'T',
    b'L',
    b'C',
    b'D',
    //b'G',
    //b'P',
    //b'M',
    //b'H',
    //b'B',
    //b'Y',
    //b'F',
    //b'V',
    //b'K',
    //b'W',
    //b'Z',
    //b'X',
    //b'J',
    //b'Q',
];

/// Holds a grouping of four three-letter subsets (`side_sets`) plus a final sequence (of length 12),
/// derived from the given `letter_pool`.
#[derive(Debug, Clone, Copy, Default)]
struct SequenceComboFilter {
  // Four sets of three letters each.
  side_sets: [LetterSet; 4],
  // The sequence of 12 letters.
  sequence: [u8; 12],
  // The letter pool from which to construct the sequence.
  letter_pool: [u8; 12],
}

impl SequenceComboFilter {
  /// Creates a new [`SequenceComboFilter`] by copying the first 12 letters from `letter_pool`.
  fn new(letter_pool: &[u8]) -> Self {
    let mut combo_filter = Self::default();
    letter_pool
      .iter()
      .zip(combo_filter.letter_pool.iter_mut())
      .for_each(|(lhs, rhs)| {
        *rhs = *lhs;
      });
    combo_filter
  }

  /// Assigns the first 3-letter `letter_set` to the first subset (index 0)
  /// and copies those letters into the front of `sequence`, zeroing them out in `letter_pool`.
  fn with_side1(mut self, letter_set: LetterSet) -> Self {
    debug_assert!(letter_set.len() == 3);
    self.side_sets[0] = letter_set;

    let mut index = 0;
    self.letter_pool.iter_mut().for_each(|letter| {
      if letter_set.has_ascii(*letter) {
        self.sequence[index] = *letter;
        index += 1;
        *letter = 0;
      }
    });

    self
  }

  /// Assigns a 3-letter `letter_set` to the second subset (index 1)
  /// and copies those letters to `sequence[3..6]`.
  fn with_side2(mut self, letter_set: LetterSet) -> Self {
    debug_assert!(letter_set.len() == 3);
    self.side_sets[1] = letter_set;

    let mut index = 3;
    self.letter_pool.iter_mut().for_each(|letter| {
      if letter_set.has_ascii(*letter) {
        self.sequence[index] = *letter;
        index += 1;
        *letter = 0;
      }
    });

    self
  }

  /// Assigns a 3-letter `letter_set` to the third subset (index 2)
  /// and copies those letters to `sequence[6..9]`.
  fn with_side3(mut self, letter_set: LetterSet) -> Self {
    debug_assert!(letter_set.len() == 3);
    self.side_sets[2] = letter_set;

    let mut index = 6;
    self.letter_pool.iter_mut().for_each(|letter| {
      if letter_set.has_ascii(*letter) {
        self.sequence[index] = *letter;
        index += 1;
        *letter = 0;
      }
    });

    self
  }

  /// Assigns a 3-letter `letter_set` to the fourth subset (index 3)
  /// and copies those letters to `sequence[9..12]`.
  fn with_side4(mut self, letter_set: LetterSet) -> Self {
    debug_assert!(letter_set.len() == 3);
    self.side_sets[3] = letter_set;

    let mut index = 9;
    self.letter_pool.iter_mut().for_each(|letter| {
      if letter_set.has_ascii(*letter) {
        self.sequence[index] = *letter;
        index += 1;
        *letter = 0;
      }
    });

    self
  }
}

impl Eq for SequenceComboFilter {}

impl PartialEq for SequenceComboFilter {
  /// Checks equality by seeing if both `side_sets` contain exactly the same subsets,
  /// ignoring the order of subsets.
  fn eq(&self, other: &Self) -> bool {
    self
      .side_sets
      .iter()
      .all(|self_set| other.side_sets.contains(self_set))
  }
}

impl Ord for SequenceComboFilter {
  /// Sorts both `side_sets` arrays and compares them lexicographically.
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let mut self_sets = self.side_sets;
    let mut other_sets = other.side_sets;

    self_sets.sort();
    other_sets.sort();

    self_sets.cmp(&other_sets)
  }
}

impl PartialOrd for SequenceComboFilter {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

/// Generates all 12-letter sequences that include the 5 vowels (`A, E, I, O, U`) plus
/// 7 selected consonants from the pool defined by `CONSONANTS`.
///
/// The function:
/// 1. Takes all 7-element combinations from `CONSONANTS`.
/// 2. Extends each combination with the 5 vowels.
/// 3. Sorts the resulting 12-letter slice.
/// 4. Returns an iterator of `Vec<u8>` for each unique 12-letter set.
fn sequences_with_all_vowels() -> impl Iterator<Item = Vec<u8>> {
  CONSONANTS
    .iter()
    .copied()
    .combinations(7)
    .zip(std::iter::repeat(VOWELS))
    .map(|(mut consonants, vowels)| {
      consonants.extend(vowels);
      consonants.sort_unstable();
      consonants
    })
}

/// For a given 12-letter slice, generates all unique ways to split it into four 3-letter subsets.
///
/// Internally, this:
/// 1. Chooses 3 letters for `side1`, storing them in `self.sequence[0..3]`.
/// 2. Chooses 3 letters for `side2` from the remaining pool, storing them in `self.sequence[3..6]`.
/// 3. Chooses 3 letters for `side3`, storing them in `self.sequence[6..9]`.
/// 4. Chooses 3 letters for `side4`, storing them in `self.sequence[9..12]`.
/// 5. Uses sorting + dedup to ensure uniqueness when the same sets are chosen in different orders.
fn all_inputs_from_sequence(sequence: &[u8]) -> impl Iterator<Item = SequenceComboFilter> + '_ {
  let one_side = sequence
    .iter()
    .copied()
    .array_combinations::<3>()
    .map(move |side1| {
      let letter_set = LetterSet::from_ascii_slice(side1.as_slice());
      let combo_filter = SequenceComboFilter::new(sequence);
      combo_filter.with_side1(letter_set)
    });

  let two_sides = one_side
    .flat_map(|combo_filter| {
      combo_filter
        .letter_pool
        .into_iter()
        .filter(|&letter| letter != 0)
        .array_combinations::<3>()
        .map(move |side2| {
          let letter_seq = LetterSet::from_ascii_slice(&side2);
          combo_filter.with_side2(letter_seq)
        })
    })
    .sorted()
    .dedup();

  let three_sides = two_sides
    .flat_map(|combo_filter| {
      combo_filter
        .letter_pool
        .into_iter()
        .filter(|&letter| letter != 0)
        .array_combinations::<3>()
        .map(move |side3| {
          let letter_seq = LetterSet::from_ascii_slice(&side3);
          combo_filter.with_side3(letter_seq)
        })
    })
    .sorted()
    .dedup();

  three_sides
    .flat_map(|combo_filter| {
      combo_filter
        .letter_pool
        .into_iter()
        .filter(|&letter| letter != 0)
        .array_combinations::<3>()
        .map(move |side4| {
          let letter_seq = LetterSet::from_ascii_slice(&side4);
          combo_filter.with_side4(letter_seq)
        })
    })
    .sorted()
    .dedup()
}

fn main() {
  let max_count = RwLock::new(0);
  let solved_count = RwLock::new(0);

  // Generate sequences that definitely include all vowels,
  // then for each sequence, generate all ways to split into four three-letter subsets.
  sequences_with_all_vowels()
    .flat_map(|sequence| all_inputs_from_sequence(sequence.as_slice()).collect::<Vec<_>>())
    .enumerate()
    .par_bridge()
    .for_each(|(n, combo_filter)| {
      // Convert the 12-letter sequence to a &str (without re-checking UTF-8 validity).
      let input = unsafe { str::from_utf8_unchecked(combo_filter.sequence.as_slice()) };

      // Create a letter group representation for verifying words.
      let letter_group = create_letter_group_function!(input);

      // Filter the global WORDS list to only those valid for the chosen letter group.
      let valid_words = &WORDS
        .iter()
        .copied()
        .filter(|word| word.is_valid_word(&letter_group))
        .collect::<Vec<_>>();

      let mut solution_count = 0;

      // Check how many valid ways exist to build up a 12-letter partition from these words.
      for &word in valid_words {
        solve_partition_once(word, &mut solution_count, valid_words);
      }

      // Update the total solved count.
      let solution_count = solution_count;
      *solved_count.write().unwrap() += 1;

      // If this combination yields a new maximum, record and print it.
      if *max_count.read().unwrap() < solution_count {
        *max_count.write().unwrap() = solution_count;
        println!(
          "{}: {}\tsolution: {}\t solved: {}",
          input,
          solution_count,
          n,
          *solved_count.read().unwrap()
        );
      }
    });
}

fn solve_partition_once(
  sequence: LetterSequence,
  solution_count: &mut u32,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => *solution_count += 1,
    11 => {}
    _ => {
      let (appendable_words, remaining_valid_words) = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .partition::<Vec<_>, _>(|word| word.can_append_to(sequence));

      appendable_words.iter().copied().for_each(|word| {
        solve_filter(
          word.append_to(sequence),
          solution_count,
          &remaining_valid_words,
        );
      });
    }
  }
}

fn solve_filter(
  sequence: LetterSequence,
  solution_count: &mut u32,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => *solution_count += 1,
    11 => {}
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
          solve_filter(
            word.append_to(sequence),
            solution_count,
            &remaining_valid_words,
          );
        });
    }
  }
}

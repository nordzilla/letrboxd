//! Defines functionality for representing a solution to a Letter Boxed puzzle,
//! i.e. the positions of word boundaries within a [`LetterSequence`] of 12 letters.

#[cfg(doc)]
use crate::LetterSequence;

use std::{fmt::Debug, ops::Range};

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

/// Encodes word boundaries for a [`LetterSequence`] as individual bits in a single [`u16`].
///
/// Each set bit in the [`Solution`] indicates a word boundary at the letter for that index.
/// That letter will be the final letter of the word before the boundary, and the first letter
/// of the word after the boundary (if there are more letters after the boundary).
///
/// # Example
///
/// ```text
/// IMPARTEDUNKS
/// 000000010001 -> IMPARTED DUNKS
/// 000001010001 -> IMPART TED DUNKS
/// 001000010001 -> IMP PARTED DUNKS
/// 001001010001 -> IMP PART TED DUNKS
/// ```
#[derive(Copy, Clone)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Solution(u16);

/// Debug prints a 16-bit binary representation of the underlying boundary bits.
impl Debug for Solution {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Solution")
      .field(&format!("{:>016b}", self.0))
      .finish()
  }
}

/// Defaults to an empty [`Solution`] with no boundaries.
impl Default for Solution {
  fn default() -> Self {
    Self::empty()
  }
}

/// Equality is defined such that two non-empty solutions are considered equal if they
/// have the same number of leading zeros in their underlying `u16` representation.
/// Any empty `Solution` is considered equal to any other empty `Solution`.
impl Eq for Solution {}

/// Partial equality follows the same rule as [`Eq`]: empty solutions are equal,
/// otherwise equality depends on the number of leading zeros in the `u16`.
impl PartialEq for Solution {
  fn eq(&self, other: &Self) -> bool {
    // Custom puzzle-specific definition of equality
    self.is_empty() || other.is_empty() || self.0.leading_zeros() == other.0.leading_zeros()
  }
}

/// Partially compares solutions by comparing their [`Solution::word_count`].
impl PartialOrd for Solution {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

/// Orders solutions based on their [`Solution::word_count`].
impl Ord for Solution {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.word_count().cmp(&other.word_count())
  }
}

impl Solution {
  /// The index of the final letter in a 12-letter sequence.
  pub const FINAL_LETTER_INDEX: usize = 11;

  /// Returns a new [`Solution`] with no word boundaries.
  #[must_use]
  #[inline]
  pub const fn empty() -> Self {
    Self(0)
  }

  /// Returns `true` if word boundaries exist in this solution.
  #[must_use]
  #[inline]
  pub const fn is_empty(self) -> bool {
    self.0 == 0
  }

  /// Returns the total number of word boundaries in this [`Solution`].
  ///
  /// # Panics
  ///
  /// Panics in debug mode if the solution has more than 5 words.
  ///
  /// Since each [`Solution`] as only 12 letters, and each word must be
  /// at least 3 letters, it is expected that this count should never exceed 5.
  #[must_use]
  #[inline]
  pub const fn word_count(self) -> u32 {
    debug_assert!(self.0.count_ones() <= 5);
    self.0.count_ones()
  }

  /// Returns a new [`Solution`] with a boundary bit set at the given `index`.
  ///
  /// # Panics
  ///
  /// Panics in debug mode when marking a boundary that is already set.
  #[must_use]
  #[inline]
  pub const fn mark(self, index: usize) -> Self {
    debug_assert!(self.0 & (1 << index) == 0);
    Self(self.0 | (1 << index))
  }

  /// Returns a new [`Solution`] with the boundary bit at the given `index` cleared.
  ///
  /// # Panics
  ///
  /// Panics in debug mode if attempting to unmark a word boundary at an `index`
  /// that was not previously marked.
  #[must_use]
  #[inline]
  pub const fn unmark(self, index: usize) -> Self {
    debug_assert!(self.is_empty() || self.0 & (1 << index) != 0);
    Self(self.0 & !(1 << index))
  }

  /// Extends the current final word boundary by shifting it one position rightward.
  #[must_use]
  #[inline]
  pub const fn extend_top_word(self) -> Self {
    let index = (u16::BITS - self.0.leading_zeros()) as usize;
    self.unmark(index.saturating_sub(1)).mark(index)
  }

  /// Returns an iterator over the ranges of letters that make up each word.
  ///
  /// Each [`Range<usize>`] runs from the start of a word (inclusive) to the boundary (inclusive).
  pub const fn word_ranges(self) -> impl Iterator<Item = Range<usize>> {
    debug_assert!(self.0 >> Solution::FINAL_LETTER_INDEX <= 1);
    WordRanges {
      solution: self.0,
      index: 0,
    }
  }
}

/// An iterator that splits a 12-letter sequence into individual word ranges
/// based on the boundary bits in a [`Solution`].
///
/// Returned by [`Solution::word_ranges`].
pub struct WordRanges {
  solution: u16,
  index: usize,
}

impl Iterator for WordRanges {
  type Item = Range<usize>;

  /// Returns the next [`Range<usize>`] of letters for the next word boundary.
  ///
  /// Once the internal bits are exhausted, returns `None`.
  fn next(&mut self) -> Option<Self::Item> {
    // Start a word at the current index
    let range_start = self.index;

    // Move the boundary bits and index one position to the right.
    self.solution >>= 1;
    self.index += 1;

    // If no bits remain after shifting, iteration is complete.
    if self.solution == 0 {
      return None;
    }

    // Keep consuming zeros until we find the next 1-bit,
    // which indicates the next boundary.
    while self.solution & 1 == 0 {
      self.solution >>= 1;
      self.index += 1;
    }

    // Return a range from our word start up to (and including) the newly found boundary.
    #[expect(clippy::range_plus_one)]
    Some(range_start..self.index + 1)
  }
}

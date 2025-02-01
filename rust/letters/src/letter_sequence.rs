//! Defines functionality to represent the sequence of submitted letters to the game board.

use crate::compress_letter;
use crate::LetterGroup;
use crate::LetterSet;
use crate::Solution;
use std::fmt::{Debug, Display};
use std::ops::RangeBounds;

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// [`LetterSequence`] is a stack-allocated vector of up to 12 uppercase [ASCII] letters represented internally by
/// a single [u64] value.
///
/// Since there are 26 letters in the English alphabet, each letter can be represented
/// uniquely with only 5 bits of data by subtracting the [ASCII] value for `'A'` from each letter.
///
/// * `'A'` is represented by `00000`
/// * `'B'` is represented by `00001`
/// * `'C'` is represented by `00010`
/// * `...`
/// * `'X'` is represented by `10111`
/// * `'Y'` is represented by `11000`
/// * `'Z'` is represented by `11001`
///
/// We can divide the [u64] into 12 sections of 5 bits, fitting up to 12 [ASCII] letters, with 4 extra bits left over.
///
/// One of the 4 extra bits is used to retain track of count of letters in the [`LetterSequence`] by maintaining a single
/// one-bit that separates not-yet-filled data from populated data.
///
/// The internal representation of the 64 bits within an empty [`LetterSequence`] will look like this:
///
/// ```text
///                                                         Length-tracker bit ╾┐
///                                                                             │
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
/// └┬┘ └──────────────────────────────────┬──────────────────────────────────┘
///  └╼ Extra unused bits                  └╼ Empty letter space
/// ```
///
/// Consider an example where the letter `'A'` is appended to the empty [`LetterSequence`] shown above.
///
/// The [ASCII] value for `'A'` is `1000001`. This [ASCII] value will be shifted to match the 5-bit
/// representation described above, making its value equal to `00000`. It will then be appended
/// to the [`LetterSequence`], shifting the position of the length-tracker bit by 5 bits as well:
///
/// ```text
///                                                         Length-tracker bit ╾┐
///                                                                             │
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
/// └┬┘ └──────────────────────────────────┬──────────────────────────────────┘ │
///  └╼ Extra unused bits                  └╼ Empty letter space          ┌─────┘
///                                                                       │
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000
/// └┬┘ └───────────────────────────────┬───────────────────────────────┘   │ A │
///  └╼ Extra unused bits               └╼ Empty letter space               └───┘
/// ```
///
/// Note that the length-tracker bit is critical for knowing that the group of `00000` to the right
/// of the bit is the letter `'A'`, whereas the group of `00000` to the left of the bit is empty space.
///
/// Now consider appending the letter `'F'` to the same [`LetterSequence`] that we just appended `'A'` to.
///
/// The [ASCII] value for `'F'` is `1000110`. This [ASCII] value will be shifted to match the 5-bit
/// representation described above, making its value equal to `00101`. It will then be appended
/// to the [`LetterSequence`], shifting the position of the length-tracker bit by 5 bits as well:
///
/// ```text
///                                                   Length-tracker bit ╾┐
///                                                                       │
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000
/// └┬┘ └───────────────────────────────┬───────────────────────────────┘ │ │ A │
///  └╼ Extra unused bits               └╼ Empty letter space       ┌─────┘ └─┬─┘
///                                                                 │   ┌─────┘
///                                                                 │ ┌─┴─┐
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000 00101
/// └┬┘ └────────────────────────────┬────────────────────────────┘   │ A │ │ F │
///  └╼ Extra unused bits            └╼ Empty letter space            └───┘ └───┘
/// ```
///
/// [ASCII]: https://en.wikipedia.org/wiki/ASCII
#[derive(Clone, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
// The single use of unsafe in this code is a function that creates a string from raw
// bytes and does not violate any constructor invariants for [`LetterSequence`] itself.
// https://rust-lang.github.io/rust-clippy/master/index.html#unsafe_derive_deserialize
#[allow(clippy::unsafe_derive_deserialize)]
pub struct LetterSequence {
  letters: u64,
  letter_set: LetterSet,
  solution: Solution,
}

impl Eq for LetterSequence {}

impl PartialEq for LetterSequence {
  fn eq(&self, other: &Self) -> bool {
    self.letters == other.letters && self.letter_set == other.letter_set
  }
}

impl Debug for LetterSequence {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("LetterSequence")
      .field("letters", &self.to_string())
      .field("letter_set", &self.letter_set.to_string())
      .field("solution", &self.solution)
      .finish()
  }
}

impl From<&str> for LetterSequence {
  fn from(letters: &str) -> Self {
    Self::new(letters)
  }
}

impl<T: AsRef<str>> PartialEq<T> for LetterSequence {
  fn eq(&self, other: &T) -> bool {
    self.to_string().eq(other.as_ref())
  }
}

impl PartialEq<LetterSequence> for &str {
  fn eq(&self, other: &LetterSequence) -> bool {
    other.eq(self)
  }
}

impl PartialEq<LetterSequence> for String {
  fn eq(&self, other: &LetterSequence) -> bool {
    other.eq(self)
  }
}

impl Default for LetterSequence {
  /// Returns an empty [`LetterSequence`].
  ///
  /// * See [`LetterSequence::empty`] for more documentation.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// assert_eq!(
  ///     LetterSequence::empty(),
  ///     LetterSequence::default(),
  /// );
  /// ```
  fn default() -> Self {
    Self::empty()
  }
}

impl LetterSequence {
  /// The number of bits it takes to represent one letter in a [`LetterSequence`].
  pub const BITS_PER_LETTER: usize = 5;

  /// The maximum count of letters that a [`LetterSequence`] can hold.
  pub const CAPACITY: usize = 12;

  /// The number of unused bits in a [`LetterSequence`].
  pub const UNUSED_BITS: usize = 3;

  /// Returns an empty [`LetterSequence`].
  ///
  /// ```text
  ///                                                         Length-tracker bit ╾┐
  ///                                                                             │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
  /// └┬┘ └──────────────────────────────────┬──────────────────────────────────┘
  ///  └╼ Extra unused bits                  └╼ Empty letter space
  /// ```
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// LetterSequence::empty();
  /// ```
  #[must_use]
  #[inline]
  pub const fn empty() -> Self {
    Self {
      letters: 1,
      letter_set: LetterSet::empty(),
      solution: Solution::empty(),
    }
  }

  /// Creates a new [`LetterSequence`] from the provided `letters` string.
  /// This will convert each character into its compressed 5-bit representation.
  ///
  /// # Panics
  ///
  /// In debug mode, this function will panic if any of the letters are not uppercase ASCII,
  /// or if the string length exceeds the capacity of 12.
  #[must_use]
  #[inline]
  pub const fn new(letters: &str) -> Self {
    debug_assert!(letters.len() <= LetterSequence::CAPACITY);

    let letters = letters.as_bytes();
    let mut sequence = Self::empty();

    macro_rules! maybe_append_letter_at_index {
      ($index:expr) => {
        if $index < letters.len() {
          let letter = letters[$index];
          debug_assert!(letter.is_ascii_uppercase());
          sequence = sequence.with_letter(letter);
        }
      };
    }

    maybe_append_letter_at_index!(00);
    maybe_append_letter_at_index!(01);
    maybe_append_letter_at_index!(02);
    maybe_append_letter_at_index!(03);
    maybe_append_letter_at_index!(04);
    maybe_append_letter_at_index!(05);
    maybe_append_letter_at_index!(06);
    maybe_append_letter_at_index!(07);
    maybe_append_letter_at_index!(08);
    maybe_append_letter_at_index!(09);
    maybe_append_letter_at_index!(10);
    maybe_append_letter_at_index!(11);

    sequence
  }

  /// Returns the count of letters in the [`LetterSequence`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// assert_eq!(LetterSequence::from("NICE").len(), 4);
  /// ```
  /// ```text
  ///                                 Length-tracker bit ╾┐  Length 4 ╾┐
  ///                                                     │ ┌──────────┴──────────┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
  /// ```
  #[must_use]
  #[inline]
  pub const fn len(self) -> usize {
    self.letter_set.len()
  }

  /// Returns the number of words in the [`LetterSequence`] as defined by the internal [`Solution`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// let word1 = LetterSequence::from("FISH");
  /// let word2 = LetterSequence::from("HOPE");
  /// assert_eq!(word2.append_to(word1).word_count(), 2);
  /// ```
  #[must_use]
  #[inline]
  pub const fn word_count(self) -> u32 {
    self.solution.word_count()
  }

  /// Returns [true] if the sequence contains no letters, otherwise [false].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// assert!(LetterSequence::default().is_empty());
  /// ```
  ///
  /// ```text
  ///                                                         Length-tracker bit ╾┐
  ///                                                                             │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1
  /// └┬┘ └──────────────────────────────────┬──────────────────────────────────┘
  ///  └╼ Extra unused bits                  └╼ Empty letter space
  /// ```
  #[must_use]
  #[inline]
  pub const fn is_empty(self) -> bool {
    self.letter_set.is_empty()
  }

  /// Returns [true] if the sequence contains 12 filled letters, otherwise [false].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// assert!(LetterSequence::from("ABCDEFGHIJKL").has_all_letters());
  /// ```
  #[must_use]
  #[inline]
  pub const fn has_all_letters(self) -> bool {
    debug_assert!(self.letter_set.len() <= 12);
    self.letters.leading_zeros() == 3
  }

  /// Returns a new [`LetterSequence`] with the given letter appended to the end.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// LetterSequence::from("NIC").with_letter(b'E');
  /// ```
  /// ```text
  ///                                       Length-tracker bit ╾┐
  ///                                                           │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010
  /// └┬┘ └────────────────────────┬──────────────────────────┘ │ │ N │ │ I │ │ C │
  ///  └╼ Extra unused bits        └╼ Empty letter space  ┌─────┘ └─┬─┘ └─┬─┘ └─┬─┘
  ///                                                     │   ┌─────┘     │     │
  ///                                                     │   │     ┌─────┘     │
  ///                                                     │   │     │     ┌─────┘
  ///                                                     │ ┌─┴─┐ ┌─┴─┐ ┌─┴─┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
  /// ```
  #[must_use]
  #[inline]
  pub const fn with_letter(self, letter: u8) -> Self {
    debug_assert!(self.len() < Self::CAPACITY);
    debug_assert!(letter.is_ascii_uppercase());
    let letter = compress_letter(letter);
    Self {
      letters: (self.letters << Self::BITS_PER_LETTER) | letter as u64,
      letter_set: self.letter_set.insert(letter),
      solution: self.solution.extend_top_word(),
    }
  }

  /// Returns a new [`LetterSequence`] that has `n` letters cut from the start of the input [`LetterSequence`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// LetterSequence::from("NICE").cut_from_start(2);
  /// ```
  /// ```text
  ///                                 Length-tracker bit ╾┐
  ///                                                     │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘ │ │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space     │ └───┘ └───┘ └─┬─┘ └─┬─┘
  ///                                                     └───────────┐   │     │
  ///                                                                 │ ┌─┴─┐ ┌─┴─┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00010 00100
  /// └┬┘ └───────────────────────────┬─────────────────────────────┘   │ C │ │ E │
  ///  └╼ Extra unused bits           └╼ Empty letter space             └───┘ └───┘
  /// ```
  #[must_use]
  #[inline]
  pub const fn cut_from_start(self, n: usize) -> Self {
    debug_assert!(n <= self.len());

    // The count of bits for the number of letters we want to remove
    //
    // Example: n = 2, count_of_bits_to_remove = 10
    //                                            count_of_bits_to_remove ╾┐
    //                                                       ┌────┴────┐
    // 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
    //                                                       │ N │ │ I │ │ C │ │ E │
    //                                                       └───┘ └───┘ └───┘ └───┘
    #[expect(clippy::cast_possible_truncation)]
    let count_of_bits_to_remove = (n * Self::BITS_PER_LETTER) as u32;

    // The count of bits that we will retain after cutting letters from the start of the sequence.
    // This includes the length-tracker bit along the bits of any letters that will not be cut.
    //
    // Example: n = 2, count_of_bits_to_remove = 10, leading_zeros = 43, count_of_bits_to_retain = 11
    //
    //                                 Length-tracker bit ╾┬╼ count_of_bits_to_retain ╾┐
    //                                                     │             ┌────┴────┐
    // 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
    // └────────────────────────┬────────────────────────┘   │ N │ │ I │ │ C │ │ E │
    //           leading_zeros ╾┘                            └───┘ └───┘ └───┘ └───┘
    let count_of_bits_to_retain =
      u64::BITS - (self.letters.leading_zeros() + count_of_bits_to_remove);

    // This is a bit mask that we will use to remove the letters from the start of the
    // sequence. It consists of all zeros for the bits that we want to remove,
    // and all ones for the bits that we want to retain.
    //
    // Example: n = 2, count_of_bits_to_remove = 10, leading_zeros = 43, count_of_bits_to_retain = 11
    //
    //              letter_removal_bit_mask ╾┐
    // ┌─────────────────────────────────────┴─────────────────────────────────────┐
    // 000 00000 00000 00000 00000 00000 00000 00000 00000 0 00000 00001 11111 11111
    let letter_removal_bit_mask = (1 << count_of_bits_to_retain) - 1;

    // This is a bit mask that we will use to ensure that the length-tracker bit is
    // added back into the sequence with a one-bit at the correct position.
    //
    // Example: n = 2, count_of_bits_to_remove = 10, leading_zeros = 43, count_of_bits_to_retain = 11
    //
    //      updated_length_tracker_bit_mask ╾┐
    // ┌─────────────────────────────────────┴─────────────────────────────────────┐
    // 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000 00000
    let updated_length_tracker_bit_mask = 1 << (count_of_bits_to_retain - 1);

    // First, we do a bitwise AND operation with the letter_removal_bit_mask.
    //                                                                     ┌───┐ ┌───┐ ┌───┐ ┌───┐
    //                                                                     │ N │ │ I │ │ C │ │ E │
    //     sequence: 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
    //         (AND)                                                     │ │││││ │││││ │││││ │││││
    //  remove mask: 000 00000 00000 00000 00000 00000 00000 00000 00000 0 00000 00001 11111 11111
    //  ─────────────────────────────────────────────────────────────────┴──┴┴─┴──┴───────│────│──
    //       result: 000 00000 00000 00000 00000 00000 00000 00000 00000 0 00000 00000 00010 00100
    //                                                                                 │ C │ │ E │
    //                                                                                 └───┘ └───┘
    // Next, we re-imagine the groupings to accommodate the new position of the length-tracker bit.
    // This is not a computational operation, only an adjustment to our mental model.
    //                                                                                 ┌───┐ ┌───┐
    //                                                                                 │ C │ │ E │
    //       result: 000 00000 00000 00000 00000 00000 00000 00000 00000 0 00000 00000 00010 00100
    //                                                                   └───────────┐
    //  re-imagined: 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 0 00010 00100
    //                                                                                 │ C │ │ E │
    //                                                                                 └───┘ └───┘
    //
    // Finally, we do a bitwise OR operation using the re-imagined result from the previous step
    // with the updated_length_tracker_bit_mask.
    //
    //  re-imagined: 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 0 00010 00100
    //          (OR)                                                                 │    │    │
    // tracker mask: 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00000 00000
    // ──────────────────────────────────────────────────────────────────────────────│────│────│──
    //       result: 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 00010 00100
    //                                                                               │ │ C │ │ E │
    //                                                           Length-tracker bit ╾┘ └───┘ └───┘
    let letters = self.letters & letter_removal_bit_mask | updated_length_tracker_bit_mask;

    Self {
      letters,
      letter_set: LetterSet::from_raw_letters(letters),
      solution: Solution::empty(),
    }
  }

  /// Returns a new [`LetterSequence`] that has `n` letters cut from the end of the input [`LetterSequence`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// LetterSequence::from("NICE").cut_from_end(2);
  /// ```
  ///
  /// **Before**
  ///
  /// ```text
  ///                                 Length-tracker bit ╾┐
  ///                                                     │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘ │ │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space     │ └─┬─┘ └─┬─┘ └───┘ └───┘
  ///                                                     │   │     └───────────┐
  ///                                                     │   └───────────┐     │
  ///                                                     └───────────┐   │     │
  ///                                                                 │ ┌─┴─┐ ┌─┴─┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000
  /// └┬┘ └───────────────────────────┬─────────────────────────────┘   │ N │ │ I │
  ///  └╼ Extra unused bits           └╼ Empty letter space             └───┘ └───┘
  /// ```
  #[must_use]
  #[inline]
  pub const fn cut_from_end(self, n: usize) -> Self {
    debug_assert!(n <= self.len());

    let letters = self.letters >> (n * Self::BITS_PER_LETTER);

    Self {
      letters,
      letter_set: LetterSet::from_raw_letters(letters),
      solution: Solution::empty(),
    }
  }

  /// Returns a new [`LetterSequence`] that is a slice of the input [`LetterSequence`] based on the [`RangeBounds`].
  ///
  /// Note that this function exists instead of the [Index] trait, because [`Index::index`] requires
  /// returning a reference, but this function takes advantage of the [Copy] property of [`LetterSequence`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// LetterSequence::from("NICE").slice(1..3);
  /// ```
  /// ```text
  ///                                 Length-tracker bit ╾┐
  ///                                                     │
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘ │ │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space     │ └───┘ └─┬─┘ └─┬─┘ └───┘
  ///                                                     │         │     └─────┐
  ///                                                     │         └─────┐     │
  ///                                                     └───────────┐   │     │
  ///                                                                 │ ┌─┴─┐ ┌─┴─┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 00000 00000 1 01000 00010
  /// └┬┘ └───────────────────────────┬─────────────────────────────┘   │ I │ │ C │
  ///  └╼ Extra unused bits           └╼ Empty letter space             └───┘ └───┘
  /// ```
  ///
  /// [Index]: std::ops::Index
  /// [Index::index]: std::ops::Index::index
  #[must_use]
  #[inline]
  pub fn slice(self, bounds: impl RangeBounds<usize>) -> Self {
    let inclusive_start_bound = match bounds.start_bound() {
      std::ops::Bound::Unbounded => 0,
      std::ops::Bound::Included(&start_bound) => start_bound,
      std::ops::Bound::Excluded(&start_bound) => start_bound.saturating_add(1),
    };
    let exclusive_end_bound = match bounds.end_bound() {
      std::ops::Bound::Unbounded => self.len(),
      std::ops::Bound::Excluded(&end_bound) => end_bound,
      std::ops::Bound::Included(&end_bound) => end_bound.saturating_add(1),
    };

    debug_assert!(inclusive_start_bound <= self.len().saturating_sub(1));
    debug_assert!(exclusive_end_bound <= self.len());

    self
      .cut_from_start(inclusive_start_bound)
      .cut_from_end(self.len() - exclusive_end_bound)
  }

  /// Returns an iterator over the letters stored in this [`LetterSequence`],
  /// yielding them in last-in-first-out order.
  ///
  /// Each letter returned is in its compressed form (i.e., the 5-bit value).
  ///
  /// # Example
  ///
  /// The following sequence would return the compressed values for 'E', 'C', 'I', 'N'.
  ///
  /// ```text
  ///                                 Length-tracker bit ╾┐  Length 4 ╾┐
  ///                                                     │ ┌──────────┴──────────┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
  /// ```
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// let compress = |byte| byte - b'A';
  /// assert_eq!(
  ///   LetterSequence::from("NICE").letters_rev().collect::<Vec<_>>(),
  ///   vec![compress(b'E'), compress(b'C'), compress(b'I'), compress(b'N')],
  /// );
  /// ```
  pub const fn letters_rev(self) -> impl Iterator<Item = u8> {
    LettersRevIter(self.letters)
  }

  /// Returns an iterator over the letters stored in this [`LetterSequence`],
  /// yielding them in first-in-first-out order.
  ///
  /// Each letter returned is returned as its decompressed ASCII byte value.
  ///
  /// # Example
  ///
  /// The following sequence would return `b'N'``, `b'C'``, `b'I'``, `b'N'`.
  ///
  /// ```text
  ///                                 Length-tracker bit ╾┐  Length 4 ╾┐
  ///                                                     │ ┌──────────┴──────────┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
  /// ```
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// assert_eq!(
  ///   LetterSequence::from("NICE").ascii_bytes().collect::<Vec<_>>(),
  ///   vec![b'N', b'I', b'C', b'E'],
  /// );
  /// ```
  pub const fn ascii_bytes(self) -> impl Iterator<Item = u8> {
    ASCIIBytesIter(self.reversed_internal_representation())
  }

  /// Returns the count of letters that two [`LetterSequence`] have in common.
  #[must_use]
  #[inline]
  pub const fn shared_letter_count(self, other: LetterSequence) -> usize {
    self.letter_set.intersection(other.letter_set).len()
  }

  /// Returns [true] if `other` can safely be appended to `self` to form
  /// a larger [`LetterSequence`] without exceeding capacity, and while sharing
  /// exactly one overlapping letter (the last letter of `self` is the first letter of `other`).
  #[must_use]
  #[inline]
  pub const fn can_append_to(self, other: LetterSequence) -> bool {
    let intersection = self.letter_set.intersection(other.letter_set);
    intersection.len() == 1
      && intersection.has(other.last_letter())
      && intersection.has(self.first_letter())
      && self.letter_set.union(other.letter_set).len() <= 12
  }

  /// Returns [true] if `other` can safely be prepended to `self` to form
  /// a larger [`LetterSequence`] without exceeding capacity, and while sharing
  /// exactly one overlapping letter (the first letter of `self` is the last letter of `other`).
  #[must_use]
  #[inline]
  pub const fn can_prepend_to(self, other: LetterSequence) -> bool {
    other.can_append_to(self)
  }

  /// Appends the letters of `other` to `self`.
  /// This will merge their letter sets and appropriately mark word boundaries in the [`Solution`].
  ///
  /// # Panics
  ///
  /// In debug mode, this will panic if [`can_append_to`](Self::can_append_to) is [false].
  #[must_use]
  #[inline]
  pub const fn append_to(self, other: LetterSequence) -> Self {
    debug_assert!(self.can_append_to(other));

    let mut sequence = self.without_length_tracker_bit();
    sequence.letters |= other.letters << ((self.len() - 1) * LetterSequence::BITS_PER_LETTER);
    sequence.letter_set = other.letter_set.union(self.letter_set);
    sequence.solution = other.solution.mark(self.len() + other.len() - 2);

    sequence
  }

  /// Prepends the letters of `other` to `self`.
  /// This will merge their letter sets and appropriately mark word boundaries in the [`Solution`].
  ///
  /// # Panics
  ///
  /// In debug mode, this will panic if [`can_append_to`](Self::can_append_to) is [false].
  #[must_use]
  #[inline]
  pub const fn prepend_to(self, other: LetterSequence) -> Self {
    debug_assert!(self.can_prepend_to(other));
    other.append_to(self)
  }

  /// Returns [true] if this sequence of letters forms a valid word, according to
  /// the letter grouping logic provided by [`LetterGroup`].
  #[must_use]
  #[inline]
  pub fn is_valid_word<F>(self, letter_group: &F) -> bool
  where
    F: Fn(u8) -> LetterGroup,
  {
    self
      .letters_rev()
      .zip(self.letters_rev().skip(1))
      .all(|(lhs, rhs)| letter_group(lhs).can_be_adjacent_to(letter_group(rhs)))
  }

  /// Returns an iterator over each word in this [`LetterSequence`],
  /// where the boundaries of each word are derived from the internal [`Solution`].
  ///
  /// Each yielded item is, itself, another [`LetterSequence`] with a single word.
  pub fn words(self) -> impl Iterator<Item = LetterSequence> {
    self
      .solution
      .word_ranges()
      .map(move |range| self.slice(range))
  }

  /// Returns a human-readable solution string for the entire sequence,
  /// inserting spaces between words as indicated by the internal [`Solution`].
  ///
  /// # Example
  ///
  /// If the [`Solution`] says there are three words in the sequence `"FISHOPEAT"`,
  /// with boundaries at `'H'` and `'E'`, then this function returns `"FISH HOPE EAT"`.
  ///
  /// ```rust
  /// # use letters::LetterSequence;
  /// let word1 = LetterSequence::from("FISH");
  /// let word2 = LetterSequence::from("HOPE");
  /// let word3 = LetterSequence::from("EAT");
  ///
  /// let sequence = word1.prepend_to(word2).prepend_to(word3);
  /// assert_eq!(sequence.solution_string(), "FISH HOPE EAT");
  /// ```
  #[must_use]
  #[inline]
  pub fn solution_string(self) -> String {
    // The length of the string plus 2x the number of spaces between words.
    // We multiply by 2 to account for the space and the duplicated letter
    // that is at the end and beginning of the words at each boundary.
    let total_len = self.len() + 2 * (self.word_count() - 1) as usize;
    let mut bytes = Vec::with_capacity(total_len);

    for (index, range) in self.solution.word_ranges().enumerate() {
      if index > 0 {
        // Insert a space before all but the first word
        bytes.push(b' ');
      }

      bytes.extend(self.slice(range).ascii_bytes());
    }

    // The bytes are guaranteed to be valid ASCII bytes that
    // are either space (b' '), or in the range (b'A'..=b'Z').
    unsafe { String::from_utf8_unchecked(bytes) }
  }

  /// Returns the byte corresponding to the first letter of the sequence.
  ///
  /// # Panics
  ///
  /// Panics in debug mode if the [`LetterSequence`] is empty.
  #[expect(clippy::cast_possible_truncation)]
  const fn first_letter(self) -> u8 {
    debug_assert!(!self.is_empty());
    (self.letters >> ((self.len() - 1) * LetterSequence::BITS_PER_LETTER)) as u8 & 0b1_1111
  }

  /// Returns the byte corresponding to the last letter of the sequence.
  ///
  /// # Panics
  ///
  /// Panics in debug mode if the [`LetterSequence`] is empty.
  #[expect(clippy::cast_possible_truncation)]
  const fn last_letter(self) -> u8 {
    debug_assert!(!self.is_empty());
    self.letters as u8 & 0b1_1111
  }

  /// A private function to remove the length-tracker bit, ensuring that the raw, compressed bytes
  /// of the [`LetterSequence`] can be easily appended or prepended to another [`LetterSequence`].
  const fn without_length_tracker_bit(self) -> Self {
    Self {
      letters: self.letters & !(1 << (u64::BITS - self.letters.leading_zeros()).saturating_sub(1)),
      ..self
    }
  }

  /// A private function that returns the reversed representation of the letters within the sequence,
  /// with the letters as the left-most bits and the length-tracker bit to the right of the letters.
  ///
  /// This is helpful for iteration in left-to-right order.
  ///
  /// # Example
  ///
  /// Consider the [`LetterSequence`] `"NICE"` internally:
  ///
  /// **Before**
  ///
  /// ```text
  ///                                 Length-tracker bit ╾┐  Length 4 ╾┐
  ///                                                     │ ┌──────────┴──────────┐
  /// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
  /// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
  ///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
  /// ```
  ///
  /// **After**
  ///
  /// ```text
  ///            ┌╼ Length 4  ┌╼ Length-tracker bit
  /// ┌──────────┴──────────┐ │
  /// 01101 01000 00010 00100 1 00000 00000 00000 00000 00000 00000 00000 00000 000
  /// │ N │ │ I │ │ C │ │ E │   └─────────────────────┬───────────────────────┘ └┬┘
  /// └───┘ └───┘ └───┘ └───┘     Empty letter space ╾┘       Extra unused bits ╾┘
  /// ```
  const fn reversed_internal_representation(self) -> u64 {
    let mut letters = self.without_length_tracker_bit().letters;

    letters <<= 1;
    letters |= 1;

    let empty_letters = LetterSequence::CAPACITY - self.len();
    let empty_letter_bits = empty_letters * LetterSequence::BITS_PER_LETTER;
    letters <<= empty_letter_bits + LetterSequence::UNUSED_BITS;

    letters
  }
}

/// [`LettersRevIter`] is an iterator that yields the letters from a [`LetterSequence`]
/// in last-in-first-out (LIFO) order, with each letter returned in **compressed form**
/// (i.e., the 5-bit representation within the sequence).
///
/// Each call to [`Iterator::next`] shifts the internal bits so that the rightmost 5 bits
/// are returned next. Once all bits to the right of the length-tracker bit are consumed,
/// iteration will end.
///
/// # Example
///
/// Consider the [`LetterSequence`] `"NICE"` internally:
///
/// ```text
///                                 Length-tracker bit ╾┐  Length 4 ╾┐
///                                                     │ ┌──────────┴──────────┐
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
/// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
/// ```
///
/// Iterating with [`letters_rev`](LetterSequence::letters_rev) will yield:
///
/// - Compressed bits for `'E'`: `01101`
/// - Compressed bits for `'C'`: `01000`
/// - Compressed bits for `'I'`: `00010`
/// - Compressed bits for `'N'`: `00100`
pub struct LettersRevIter(u64);

impl Iterator for LettersRevIter {
  type Item = u8;

  #[expect(clippy::cast_possible_truncation)]
  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 1 {
      return None;
    }

    let next = self.0 as u8 & 0b1_1111;
    self.0 >>= LetterSequence::BITS_PER_LETTER;

    Some(next)
  }
}

/// [`ASCIIBytesIter`] is an iterator that yields the letters from a [`LetterSequence`]
/// in first-in-first-out (FIFO) order, **decompressed** into standard ASCII byte values
/// (`b'A'` through `b'Z'`).
///
/// This algorithm
/// Under the hood, this iterator skips over the length-tracker bit, accounts for any unused
/// extra bits, then reads each 5-bit segment from left to right. These 5-bit segments are
/// then converted back into their ASCII counterparts (by adding `b'A'`).
///
/// # Example
///
/// Consider the [`LetterSequence`] `"NICE"` internally:
///
/// ```text
///                                 Length-tracker bit ╾┐  Length 4 ╾┐
///                                                     │ ┌──────────┴──────────┐
/// 000 00000 00000 00000 00000 00000 00000 00000 00000 1 01101 01000 00010 00100
/// └┬┘ └─────────────────────┬───────────────────────┘   │ N │ │ I │ │ C │ │ E │
///  └╼ Extra unused bits     └╼ Empty letter space       └───┘ └───┘ └───┘ └───┘
/// ```
///
/// Iterating with [`ascii_bytes`](LetterSequence::ascii_bytes) will yield the ASCII bytes:
///
/// - `b'N'`
/// - `b'I'`
/// - `b'C'`
/// - `b'E'`
pub struct ASCIIBytesIter(u64);

impl Iterator for ASCIIBytesIter {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 1 << 63 {
      return None;
    }

    let next = (self.0 >> (64 - LetterSequence::BITS_PER_LETTER)) as u8;
    self.0 <<= LetterSequence::BITS_PER_LETTER;

    Some(next + b'A')
  }
}

impl Display for LetterSequence {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for byte in self.ascii_bytes() {
      write!(f, "{}", byte as char)?;
    }

    Ok(())
  }
}

#[test]
fn first_letter() {
  let letters = "ABCDEFGHIJKL";
  let compress = |letter| letter - b'A';
  for n in 1..=letters.len() {
    let sequence = LetterSequence::new(&letters[0..n]);
    assert_eq!(sequence.first_letter(), compress(b'A'));
  }
}

#[test]
fn last_letter() {
  let letters = "ABCDEFGHIJKL";
  let compress = |letter| letter - b'A';
  for n in 1..=letters.len() {
    let sequence = LetterSequence::new(&letters[0..n]);
    assert_eq!(sequence.last_letter(), compress(letters.as_bytes()[n - 1]));
  }
}

#[test]
fn without_length_tracker_bit() {
  let letters = "ABCDEFGHIJKL";
  for n in 0..=letters.len() {
    let sequence = LetterSequence::new(&letters[0..n]);
    assert!(
      sequence.letters.leading_zeros()
        < sequence
          .without_length_tracker_bit()
          .letters
          .leading_zeros(),
      "Removing the length-tracker bit should increase the leading-zero count.",
    );
  }
}

#[test]
#[expect(clippy::unusual_byte_groupings)]
fn reversed_internal_representation() {
  assert_eq!(
    LetterSequence::from("NICE").reversed_internal_representation(),
    0b_01101_01000_00010_00100_1_00000_00000_00000_00000_00000_00000_00000_00000_000,
    // │ N │ │ I │ │ C │ │ E │   └─────────────────────┬───────────────────────┘ └┬┘
    // └───┘ └───┘ └───┘ └───┘     Empty letter space ╾┘       Extra unused bits ╾┘
  );
}

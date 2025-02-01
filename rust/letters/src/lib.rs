//! The low-level optimized functionality for handling sequences of letters.

#![expect(clippy::zero_prefixed_literal)]
#![warn(missing_docs)]

pub mod letter_group;
pub mod letter_sequence;
pub mod letter_set;
pub mod solution;

pub use letter_group::LetterGroup;
pub use letter_sequence::LetterSequence;
pub use letter_set::LetterSet;
pub use solution::Solution;

/// Compresses an ASCII byte to the 5-bit format used by [`LetterSequence`]
/// by subtracting the value of `b'A'`.
#[must_use]
#[inline]
pub const fn compress_letter(ascii_byte: u8) -> u8 {
  debug_assert!(ascii_byte >= b'A');
  debug_assert!(ascii_byte <= b'Z');

  ascii_byte - b'A'
}

/// Decompresses an ASCII byte from 5-bit format used by [`LetterSequence`]
/// by adding the value of `b'A'`.
#[must_use]
#[inline]
pub const fn decompress_letter(ascii_byte: u8) -> u8 {
  debug_assert!(ascii_byte <= compress_letter(b'Z'));

  ascii_byte + b'A'
}

//! Defines functionality for a compact bitset of uppercase ASCII letters.

use std::fmt::Debug;
use std::fmt::Display;

use crate::compress_letter;
use crate::LetterSequence;

/// [`LetterSet`] is a compact bitset representing uppercase ASCII letters
/// using a single [u32]. Each of the 26 letters corresponds to a value in
/// the bit set with 6 bits of unused space left over.
///
/// ```text
/// 000000_00000000000000000000000000
/// unused ZYXWVUTSRQPONMLKJIHGFEDCBA
/// ```
///
/// [`LetterSet`] is immutable by design, and inserting a letter into the
/// set returns a new set, leaving the original intact. There is no way to
/// remove a letter from the set, since that functionality is not needed.
///
/// # Example
///
/// ```rust
/// # use letters::LetterSet;
///
/// let empty_set = LetterSet::empty();
/// let compress = |letter| letter - b'A';
///
/// let set_with_e = empty_set.insert(compress(b'E'));
///
/// assert!(empty_set.is_empty());
/// assert!(!set_with_e.is_empty());
/// assert!(set_with_e.has(compress(b'E')));
/// ```
#[derive(Clone, Copy, Default, PartialOrd, Ord)]
pub struct LetterSet(u32);

impl Debug for LetterSet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("LetterSet").field(&self.to_string()).finish()
  }
}

impl Display for LetterSet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for n in 0..26 {
      if self.0 & (1 << n) != 0 {
        write!(f, "{}", (n + b'A') as char)?;
      }
    }
    write!(f, "]")
  }
}

impl Eq for LetterSet {}

impl PartialEq for LetterSet {
  fn eq(&self, other: &Self) -> bool {
    Self::eq(*self, *other)
  }
}

impl LetterSet {
  /// Returns an empty [`LetterSet`] with no letters included.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let letters = LetterSet::empty();
  /// assert!(letters.is_empty());
  /// ```
  #[must_use]
  #[inline]
  pub const fn empty() -> Self {
    Self(0)
  }

  /// Compares two [`LetterSet`] instances for equality.
  ///
  /// This method returns [true] if both sets represent exactly the same letters.
  /// This is implemented explicitly as a `const` [fn] because [`PartialEq::eq`](Self::eq)
  /// cannot be `const`.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let lhs = LetterSet::empty().insert(0); // contains 'A'
  /// let rhs = LetterSet::empty().insert(0); // contains 'A'
  ///
  /// assert!(lhs.eq(rhs));
  /// ```
  #[must_use]
  #[inline]
  pub const fn eq(self, other: Self) -> bool {
    self.0 == other.0
  }

  /// Constructs a new [`LetterSet`] from the raw internal representation of
  /// the letters within a [`LetterSequence`].
  #[must_use]
  #[inline]
  pub const fn from_raw_letters(mut letters: u64) -> Self {
    let mut letter_set = Self::empty();

    while letters != 1 {
      letter_set = letter_set.insert((letters & 0b1_1111) as u8);
      letters >>= LetterSequence::BITS_PER_LETTER;
    }

    letter_set
  }

  /// Constructs a [`LetterSet`] from a slice of ASCII bytes.
  ///
  /// # Panics
  ///
  /// Panics in debug mode if one if any byte is not in the range from `b'A'..=b'Z'`.
  #[must_use]
  pub fn from_ascii_slice(letters: &[u8]) -> Self {
    letters
      .iter()
      .copied()
      .fold(LetterSet::empty(), |letter_set, letter| {
        letter_set.insert(compress_letter(letter))
      })
  }

  /// Returns the count of letters contained in the set.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let set = LetterSet::empty()
  ///   .insert(compress(b'H'))
  ///   .insert(compress(b'I'));
  ///
  /// assert_eq!(set.len(), 2);
  /// ```
  #[must_use]
  #[inline]
  pub const fn len(self) -> usize {
    self.0.count_ones() as usize
  }

  /// Returns [true] if the set contains no letters, otherwise [false].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let set = LetterSet::empty();
  /// assert!(set.is_empty());
  ///
  /// let set = set.insert(0);
  /// assert!(!set.is_empty());
  /// ```
  #[must_use]
  #[inline]
  pub const fn is_empty(self) -> bool {
    self.len() == 0
  }

  /// Returns [true] if the given compressed `letter` is in this [`LetterSet`], otherwise [false].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let set = LetterSet::empty()
  ///   .insert(compress(b'H'))
  ///   .insert(compress(b'I'));
  ///
  /// assert!(set.has(compress(b'H')));
  /// assert!(set.has(compress(b'I')));
  /// assert!(!set.has(compress(b'P')));
  /// ```
  #[must_use]
  #[inline]
  pub const fn has(self, letter: u8) -> bool {
    self.0 & 1 << letter > 0
  }

  /// Returns [true] if the given ASCII `letter` is in this [`LetterSet`], otherwise [false].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let set = LetterSet::empty()
  ///   .insert(compress(b'H'))
  ///   .insert(compress(b'I'));
  ///
  /// assert!(set.has(compress(b'H')));
  /// assert!(set.has(compress(b'I')));
  /// assert!(!set.has(compress(b'P')));
  /// ```
  #[must_use]
  #[inline]
  pub fn has_ascii(self, letter: u8) -> bool {
    letter.is_ascii_uppercase() && self.0 & 1 << compress_letter(letter) > 0
  }

  /// Returns a new [`LetterSet`] with the given compressed `letter` added.
  ///
  /// # Panics
  ///
  /// In debug mode, this function will panic if:
  /// * `letter` is not within the compressed-value range of A through Z.
  /// * `letter` is already present in this [`LetterSet`].
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let empty = LetterSet::empty();
  /// let set = empty.insert(compress(b'A'));
  ///
  /// assert!(empty.is_empty());
  /// assert!(!set.is_empty());
  ///
  /// assert!(set.has(compress(b'A')));
  /// assert!(!empty.has(compress(b'A')));
  /// ```
  #[must_use]
  #[inline]
  pub const fn insert(self, letter: u8) -> Self {
    debug_assert!(
      letter <= compress_letter(b'Z'),
      "The letter should be within range A through Z."
    );
    debug_assert!(
      0 == (self.0 & (1 << letter)),
      "The set should not already contain the letter."
    );

    Self(self.0 | 1 << letter)
  }

  /// Returns a new [`LetterSet`] that contains only the letters present in both
  /// `self` and `other`.
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let lhs = LetterSet::empty()
  ///   .insert(compress(b'A'))
  ///   .insert(compress(b'B'))
  ///   .insert(compress(b'C'));
  ///
  /// let rhs = LetterSet::empty()
  ///   .insert(compress(b'A'))
  ///   .insert(compress(b'B'))
  ///   .insert(compress(b'D'));
  ///
  /// let intersection = lhs.intersection(rhs);
  ///
  /// assert!(intersection.has(compress(b'A')));
  /// assert!(intersection.has(compress(b'B')));
  /// assert!(!intersection.has(compress(b'C')));
  /// assert!(!intersection.has(compress(b'D')));
  /// ```
  #[must_use]
  #[inline]
  pub const fn intersection(self, other: LetterSet) -> LetterSet {
    Self(self.0 & other.0)
  }

  /// Returns a new [`LetterSet`] that contains the letters present in either
  /// `self` or `other` (or both).
  ///
  /// # Example
  ///
  /// ```rust
  /// # use letters::LetterSet;
  /// let compress = |letter| letter - b'A';
  ///
  /// let lhs = LetterSet::empty()
  ///   .insert(compress(b'A'))
  ///   .insert(compress(b'B'))
  ///   .insert(compress(b'C'));
  ///
  /// let rhs = LetterSet::empty()
  ///   .insert(compress(b'A'))
  ///   .insert(compress(b'B'))
  ///   .insert(compress(b'D'));
  ///
  /// let union = lhs.union(rhs);
  ///
  /// assert!(union.has(compress(b'A')));
  /// assert!(union.has(compress(b'B')));
  /// assert!(union.has(compress(b'C')));
  /// assert!(union.has(compress(b'D')));
  /// ```
  #[must_use]
  #[inline]
  pub const fn union(self, other: LetterSet) -> LetterSet {
    Self(self.0 | other.0)
  }

  /// Returns an iterator over the ASCII bytes contained with this [`LetterSet`].
  #[must_use]
  pub fn ascii_bytes(self) -> AsciiBytes {
    AsciiBytes {
      current_letter: b'A',
      letter_set: self,
    }
  }
}

/// An iterator over the ASCII bytes contained within a [`LetterSet`].
pub struct AsciiBytes {
  current_letter: u8,
  letter_set: LetterSet,
}

impl Iterator for AsciiBytes {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.letter_set.is_empty() {
      return None;
    }

    while self.letter_set.0 & 1 != 1 {
      self.letter_set.0 >>= 1;
      self.current_letter += 1;
    }

    self.letter_set.0 >>= 1;
    self.current_letter += 1;

    Some(self.current_letter - 1)
  }
}

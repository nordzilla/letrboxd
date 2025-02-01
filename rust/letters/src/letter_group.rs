//! Defines a way to group the letters of the four input sides of a Letter Boxed puzzle.

/// Creates a closure that classifies letters into one of four groups or marks
/// them as invalid. Each group corresponds to one of the three letters on each
/// side of a Letter Boxed puzzle input.
///
/// # Panics
///
/// Panics in debug mode if the provided string does not have exactly 12 characters.
///
/// # Example
///
/// ```
/// # use letters::create_letter_group_function;
/// # use letters::letter_group::LetterGroup;
/// // "ABC" -> Group1, "DEF" -> Group2, "GHI" -> Group3, "JKL" -> Group4
/// let letter_group = create_letter_group_function!("ABCDEFGHIJKL");
/// let compress = |letter| letter - b'A';
///
/// assert_eq!(letter_group(compress(b'A')), LetterGroup::Group1);
/// assert_eq!(letter_group(compress(b'E')), LetterGroup::Group2);
/// assert_eq!(letter_group(compress(b'L')), LetterGroup::Group4);
/// assert_eq!(letter_group(compress(b'X')), LetterGroup::Invalid);
/// ```
#[macro_export]
macro_rules! create_letter_group_function {
  ($str:expr) => {{
    debug_assert!($str.len() == 12);
    let &[a0, a1, a2, b0, b1, b2, c0, c1, c2, d0, d1, d2] = $str.as_bytes() else {
      panic!(
        r#"Expected input to letter_group function, "{}", to have exactly 12 characters."#,
        $str
      );
    };
    $crate::create_letter_group_function!(
      [
        $crate::compress_letter(a0),
        $crate::compress_letter(a1),
        $crate::compress_letter(a2)
      ],
      [
        $crate::compress_letter(b0),
        $crate::compress_letter(b1),
        $crate::compress_letter(b2)
      ],
      [
        $crate::compress_letter(c0),
        $crate::compress_letter(c1),
        $crate::compress_letter(c2)
      ],
      [
        $crate::compress_letter(d0),
        $crate::compress_letter(d1),
        $crate::compress_letter(d2)
      ],
    )
  }};
  (
        [$a0:expr, $a1:expr, $a2:expr],
        [$b0:expr, $b1:expr, $b2:expr],
        [$c0:expr, $c1:expr, $c2:expr],
        [$d0:expr, $d1:expr, $d2:expr],
    ) => {{
    move |letter| {
      if $a0 == letter || $a1 == letter || $a2 == letter {
        return $crate::letter_group::LetterGroup::Group1;
      } else if $b0 == letter || $b1 == letter || $b2 == letter {
        return $crate::letter_group::LetterGroup::Group2;
      } else if $c0 == letter || $c1 == letter || $c2 == letter {
        return $crate::letter_group::LetterGroup::Group3;
      } else if $d0 == letter || $d1 == letter || $d2 == letter {
        return $crate::letter_group::LetterGroup::Group4;
      }

      $crate::letter_group::LetterGroup::Invalid
    }
  }};
}

/// Represents possible group classifications for a given letter.
///
/// - [`Invalid`]: The letter does not fit any of the four defined groups.
/// - [`Group1`], [`Group2`], [`Group3`], [`Group4`]: Each variant indicates that the
///   letter belongs to one of four different categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetterGroup {
  /// A letter that does not belong to any defined group.
  Invalid,
  /// A letter from the first group.
  Group1,
  /// A letter from the second group.
  Group2,
  /// A letter from the third group.
  Group3,
  /// A letter from the fourth group.
  Group4,
}

impl LetterGroup {
  /// Determines whether this group can be adjacent to `other`.
  ///
  /// [`LetterGroup::Invalid`] cannot be adjacent to anything.
  /// Each other group type can only be adjacent to a group of
  /// a different type than its own type.
  ///
  /// # Example
  ///
  /// ```
  /// # use letters::letter_group::LetterGroup;
  /// assert!(LetterGroup::Group1.can_be_adjacent_to(LetterGroup::Group2));
  /// assert!(!LetterGroup::Group1.can_be_adjacent_to(LetterGroup::Group1));
  /// assert!(!LetterGroup::Group2.can_be_adjacent_to(LetterGroup::Invalid));
  /// ```
  #[must_use]
  #[inline]
  pub const fn can_be_adjacent_to(self, other: Self) -> bool {
    use LetterGroup::{Group1, Group2, Group3, Group4, Invalid};
    !matches!(
      (self, other),
      (_, Invalid)
        | (Invalid, _)
        | (Group1, Group1)
        | (Group2, Group2)
        | (Group3, Group3)
        | (Group4, Group4)
    )
  }
}

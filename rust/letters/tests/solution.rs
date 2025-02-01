use std::ops::Range;

use letters::Solution;

#[test]
fn empty() {
  assert!(Solution::empty().is_empty(), "An empty solution is empty.");

  assert_eq!(
    0,
    Solution::empty().word_count(),
    "An empty solution has a word count of zero.",
  );

  assert!(
    Solution::empty().word_ranges().next().is_none(),
    "An empty solution produces no word ranges.",
  );
}

#[test]
fn is_empty() {
  assert!(Solution::empty().is_empty(), "An empty solution is empty.");

  assert!(
    !Solution::empty().mark(2).is_empty(),
    "A solution with a marked index is not empty.",
  );
}

#[test]
fn word_count() {
  assert_eq!(
    0,
    Solution::empty().word_count(),
    "An empty solution has a word count of zero.",
  );

  let mut solution = Solution::empty();

  for (index, expected_word_count) in [2, 4, 6, 8, 11].into_iter().zip(1..) {
    solution = solution.mark(index);
    assert_eq!(
      expected_word_count,
      solution.word_count(),
      "The solution has the expected word count.",
    );
  }
}

#[test]
fn mark() {
  assert!(
    !Solution::empty().mark(2).is_empty(),
    "A solution with a marked index is not empty.",
  );

  let mut solution = Solution::empty();

  for (index, expected_word_count) in [2, 4, 6, 8, 11].into_iter().zip(1..) {
    solution = solution.mark(index);
    assert_eq!(
      expected_word_count,
      solution.word_count(),
      "The solution has the expected word count.",
    );
  }
}

#[test]
fn unmark() {
  assert!(
    Solution::empty().unmark(0).is_empty(),
    "Unmarking an empty solution returns an empty solution.",
  );

  assert!(
    Solution::empty().mark(2).unmark(2).is_empty(),
    "A solution with an index marked, then unmarked,is empty.",
  );

  assert_eq!(
    Solution::empty().mark(7).mark(9),
    Solution::empty().mark(7).mark(8).mark(9).unmark(8),
    "Unmarking a value from a solution affects only that value.",
  );
}

#[test]
fn extend_top_word() {
  assert_eq!(
    Solution::empty().mark(0),
    Solution::empty().extend_top_word(),
    "Extending the top word of an empty solution is equivalent to marking the zero index.",
  );

  for n in 0..15 {
    assert_eq!(
            Solution::empty().mark(n + 1),
            Solution::empty().mark(n).extend_top_word(),
            "Extending the top word of a solution with index {n} marked, should be equivalent to marking index {}",
            n + 1,
        );
  }

  for n in 1..15 {
    assert_eq!(
      Solution::empty().mark(n - 1).mark(n + 1),
      Solution::empty().mark(n - 1).mark(n).extend_top_word(),
      "Extending the the top word does not affect the mark below the top word",
    );
  }

  for n in 3..15 {
    assert_eq!(
      Solution::empty().mark(n - 3).mark(n - 1).mark(n + 1),
      Solution::empty()
        .mark(n - 3)
        .mark(n - 1)
        .mark(n)
        .extend_top_word(),
      "Extending the the top word does not affect multiple marks below the top word",
    );
  }
}

#[test]
fn word_ranges() {
  assert_eq!(
    Vec::<Range<usize>>::new(),
    Solution::empty().word_ranges().collect::<Vec<_>>(),
    "An empty solution produces no word ranges.",
  );

  assert_eq!(
    vec![0..3],
    Solution::empty().mark(2).word_ranges().collect::<Vec<_>>(),
    "A solution with one marked index produces one word range.",
  );

  let expected = vec![0..3, 2..5, 4..7, 6..9, 8..12];

  let actual = Solution::empty()
    .mark(2)
    .mark(4)
    .mark(6)
    .mark(8)
    .mark(11)
    .word_ranges()
    .collect::<Vec<_>>();

  assert_eq!(expected, actual,);
}

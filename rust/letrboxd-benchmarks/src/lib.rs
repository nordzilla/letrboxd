use letters::{create_letter_group_function, LetterSequence};
use word_list::WORDS;

pub const TEST_INPUT: &str = "EIONRSTDGLAU";
pub const TEST_INPUT_SOLUTION_COUNT: usize = 351_535;

pub fn count_solutions<F>(input: &str, solve: F) -> usize
where
  F: Fn(LetterSequence, &mut Vec<LetterSequence>, &[LetterSequence]),
{
  let letter_group = create_letter_group_function!(input);

  let valid_words = &WORDS
    .iter()
    .copied()
    .filter(|word| word.is_valid_word(&letter_group))
    .collect::<Vec<_>>();

  let solutions = &mut Vec::new();

  for &word in valid_words {
    solve(word, solutions, valid_words);
  }

  solutions.len()
}

pub fn solve_filter_only(
  sequence: LetterSequence,
  solutions: &mut Vec<LetterSequence>,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => solutions.push(sequence),
    11 => (),
    _ => {
      let valid_words = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .collect::<Vec<_>>();
      valid_words
        .iter()
        .copied()
        .filter(|word| word.can_append_to(sequence))
        .for_each(|word| {
          solve_filter_only(word.append_to(sequence), solutions, &valid_words);
        });
    }
  }
}

pub fn solve_partition(
  sequence: LetterSequence,
  solutions: &mut Vec<LetterSequence>,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    // If we have constructed a valid sequence with exactly 12 letters, it is a solution.
    12 => {
      solutions.push(sequence);
    }
    // An 11-letter sequence cannot form a valid 12-letter solution given
    // that the minimum word length is 3 letters.
    11 => {}
    _ => {
      let (appendable_words, remaining_valid_words) = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .partition::<Vec<_>, _>(|word| word.can_append_to(sequence));
      appendable_words.iter().copied().for_each(|word| {
        solve_partition(word.append_to(sequence), solutions, &remaining_valid_words);
      });
    }
  }
}

pub fn solve_partition_once(
  sequence: LetterSequence,
  solutions: &mut Vec<LetterSequence>,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    // If we have constructed a valid sequence with exactly 12 letters, it is a solution.
    12 => {
      solutions.push(sequence);
    }
    // An 11-letter sequence cannot form a valid 12-letter solution given
    // that the minimum word length is 3 letters.
    11 => {}
    _ => {
      let (appendable_words, remaining_valid_words) = valid_words
        .iter()
        .copied()
        .filter(|word| word.shared_letter_count(sequence) <= 1)
        .partition::<Vec<_>, _>(|word| word.can_append_to(sequence));
      appendable_words.iter().copied().for_each(|word| {
        solve_filter_only(word.append_to(sequence), solutions, &remaining_valid_words);
      });
    }
  }
}

#[cfg(test)]
mod test {
  use crate::*;
  #[test]
  fn filter_only() {
    assert_eq!(
      TEST_INPUT_SOLUTION_COUNT,
      count_solutions(TEST_INPUT, solve_filter_only),
    );
  }

  #[test]
  fn partition() {
    assert_eq!(
      TEST_INPUT_SOLUTION_COUNT,
      count_solutions(TEST_INPUT, solve_partition),
    );
  }

  #[test]
  fn partition_once() {
    assert_eq!(
      TEST_INPUT_SOLUTION_COUNT,
      count_solutions(TEST_INPUT, solve_partition_once),
    );
  }
}

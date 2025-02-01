use crossbeam::thread;
use letters::{create_letter_group_function, LetterSequence};
use std::env;
use word_list::WORDS;

fn main() {
  let args = env::args().collect::<Vec<_>>();
  let input = &args[1];
  let letter_group = create_letter_group_function!(input);

  let valid_words = &WORDS
    .iter()
    .copied()
    .filter(|word| word.is_valid_word(&letter_group))
    .collect::<Vec<_>>();

  let mut buckets = vec![Vec::new(); num_cpus::get()];
  let words = valid_words
    .chunks(valid_words.len() / buckets.len() + 1)
    .collect::<Vec<_>>();

  let _ = thread::scope(|s| {
    buckets.iter_mut().zip(words).for_each(|(bucket, words)| {
      s.spawn(move |_| {
        for &word in words {
          solve_partition_once(word, bucket, valid_words);
        }
      });
    });
  });

  let mut solutions = buckets.into_iter().flatten().collect::<Vec<_>>();
  let solution_count = solutions.len();

  solutions.sort_by_key(|solution| solution.word_count());

  for solution in solutions {
    for word in solution.words() {
      print!("{word} ");
    }
    println!();
  }

  println!("\n\n{solution_count} solutions");
}

fn solve_partition_once(
  sequence: LetterSequence,
  solutions: &mut Vec<LetterSequence>,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => solutions.push(sequence),
    11 => (),
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

fn solve_filter(
  sequence: LetterSequence,
  solutions: &mut Vec<LetterSequence>,
  valid_words: &[LetterSequence],
) {
  match sequence.len() {
    12 => solutions.push(sequence),
    11 => (),
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

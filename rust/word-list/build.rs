//! This module processes the raw Wordnik word-list file and generates a Rust source file
//! containing a static array of valid words as [`LetterSequence`] instances.
//!
//! This build script maintains a CRC hash over the word-list data, only regenerating the
//! static array if the word list has changed from the previous build.

use crc32fast::Hasher;
use std::collections::BTreeSet;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

#[cfg(doc)]
use letters::LetterSequence;

static ALL_WORDS_CRC_PATH: &str = "data/all_words.crc";
static ALL_WORDS_SOURCE_PATH: &str = "data/all_words.txt";
static VALID_WORDS_OUTPUT_PATH: &str = "src/generated/words.rs";

/// Checks if a word has all unique letters.
fn has_unique_letters(word: &str) -> bool {
  let mut unique_chars = BTreeSet::new();
  word.chars().all(|c| unique_chars.insert(c))
}

/// Calculates the CRC32 hash of the contents of the given file.
fn calculate_file_hash<P: AsRef<Path>>(path: P) -> io::Result<u32> {
  let bytes = &mut Vec::new();
  File::open(path)?.read_to_end(bytes)?;

  let mut hasher = Hasher::new();
  hasher.update(bytes);

  Ok(hasher.finalize())
}

/// Saves the given hash value to the predefined hash file.
fn save_hash(hash: u32) -> io::Result<()> {
  let mut file = File::create(ALL_WORDS_CRC_PATH)?;
  write!(file, "{hash:08X}")?;
  Ok(())
}

/// Loads the stored CRC32 hash from the predefined hash file, if it exists.
fn load_hash() -> io::Result<Option<u32>> {
  let path = Path::new(ALL_WORDS_CRC_PATH);

  if !path.exists() {
    return Ok(None);
  }

  let mut file = File::open(path)?;
  let hash = &mut String::new();

  file.read_to_string(hash)?;
  let hash = u32::from_str_radix(hash.trim(), 16)
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid hash format"))?;

  Ok(Some(hash))
}

/// Reads the source word file and filters valid words based on the following criteria that would make
/// them compatible to exist within a unique-letter solution to a Letter Boxed puzzle.
///
/// - The length of the word is in range 3..11, or exactly 12.
/// - All letters in the word are unique.
fn valid_words() -> std::io::Result<Vec<String>> {
  let file = File::open(ALL_WORDS_SOURCE_PATH)?;
  let reader = BufReader::new(file);

  let mut valid_words = Vec::new();

  let lines = reader
    .lines()
    .skip_while(|line| line.is_err() || line.as_ref().is_ok_and(|line| line.starts_with("//")))
    .flatten();

  for word in lines {
    let len = word.len();

    if ((3..11).contains(&len) || len == 12) && has_unique_letters(&word) {
      valid_words.push(word);
    }
  }

  Ok(valid_words)
}

/// Processes the word list by generating a Rust source file containing
/// a static array of [`LetterSequence`] representing the valid words.
fn process_word_list() -> std::io::Result<()> {
  let file = &mut File::create(VALID_WORDS_OUTPUT_PATH)?;
  let valid_words = valid_words()?;

  writeln!(file, "use letters::LetterSequence;")?;
  writeln!(file)?;
  writeln!(file, "#[rustfmt::skip]")?;
  writeln!(file, "#[allow(long_running_const_eval)]")?;
  writeln!(file, "pub static WORDS: &[LetterSequence] = &[")?;

  for word in valid_words {
    writeln!(
      file,
      r#"    LetterSequence::new("{}"{:>pad$}),"#,
      word.to_ascii_uppercase(),
      "",
      pad = 12 - word.len()
    )?;
  }

  writeln!(file, "];")
}

fn main() -> std::io::Result<()> {
  let file_hash = calculate_file_hash(ALL_WORDS_SOURCE_PATH)?;

  if let Some(stored_hash) = load_hash()? {
    if file_hash == stored_hash {
      // The file has not changed, nothing to do.
      return Ok(());
    }
  }

  process_word_list()?;
  save_hash(file_hash)
}

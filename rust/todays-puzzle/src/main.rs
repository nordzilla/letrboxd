pub mod puzzle_input;

use puzzle_input::{DatesByInput, InputsByDate, PuzzleInput};
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::path::PathBuf;

/// Fetch today's puzzle input from the official NYT site.
/// This code remains as-is, using the data from `window.gameData`.
fn fetch_todays_puzzle_input() -> Result<PuzzleInput, Box<dyn Error>> {
  let html = get("https://www.nytimes.com/puzzles/letter-boxed")?.text()?;
  let document = Html::parse_document(&html);
  let script_selector = Selector::parse("script")?;
  let game_data_regex = Regex::new(r"window\.gameData\s*?=\s*?(\{.*?\})")?;

  for script in document.select(&script_selector) {
    for text in script.text() {
      if let Some(captures) = game_data_regex.captures(text) {
        let game_data = &captures[1];
        let json: Value = serde_json::from_str(game_data)?;
        let puzzle_input = PuzzleInput::try_from(&json)?;
        return Ok(puzzle_input);
      }
    }
  }

  Err("Failed to retrieve data for today's puzzle.".into())
}

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();

  // Expect at least 2 arguments:
  //   - 0: the executable name
  //   - 1: the output directory
  if args.len() < 2 {
    eprintln!("Usage: {} <output_dir>", args[0]);
    std::process::exit(1);
  }

  let output_dir = &args[1];

  // Check that the provided output_dir exists and is a directory.
  let path = PathBuf::from(output_dir);
  if !path.exists() {
    eprintln!("The path '{}' does not exist.", path.to_string_lossy());
    std::process::exit(1);
  }
  if !path.is_dir() {
    eprintln!("The path '{}' is not a directory.", path.to_string_lossy());
    std::process::exit(1);
  }

  let puzzle_input = fetch_todays_puzzle_input()?;

  // Load or create data files.
  let mut inputs_by_date = InputsByDate::read_from_file_or_create(&path);
  let mut dates_by_input = DatesByInput::read_from_file_or_create(&path);

  // Insert the puzzle data.
  inputs_by_date.insert(&puzzle_input);
  dates_by_input.insert(&puzzle_input);

  // Write to files.
  inputs_by_date.write_to_file(&path)?;
  dates_by_input.write_to_file(&path)?;

  Ok(())
}

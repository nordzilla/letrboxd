use std::cmp::Reverse;
use std::{collections::BTreeMap, error::Error, fs::File, path::Path};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const INPUTS_BY_DATE: &str = "inputsByDate.json";
pub const DATES_BY_INPUT: &str = "datesByInput.json";

/// A mapping from the puzzle’s publication date (wrapped in a `Reverse` to sort descending)
/// to the puzzle input (a 12-character string composed of 4 three-letter sides).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InputsByDate(BTreeMap<Reverse<NaiveDate>, String>);

/// A mapping from a puzzle input (in a normalized form) to the puzzle’s publication date.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DatesByInput(BTreeMap<String, NaiveDate>);

/// Represents the data for a single puzzle, containing its publication date
/// and the original 12-character input string.
#[derive(Serialize, Deserialize)]
pub struct PuzzleInput {
  /// Date of the puzzle in `YYYY-MM-DD` format.
  pub date: NaiveDate,
  /// The puzzle's 12-character input, derived by concatenating four 3-letter sides.
  pub input: String,
}

impl PuzzleInput {
  /// Returns a normalized version of the puzzle input.
  ///
  /// This takes the puzzle’s 12-character string, splits it into four chunks of three letters,
  /// sorts each chunk’s letters, and concatenates them back together.
  ///
  /// # Examples
  ///
  /// If the `input` is `"CABXYZPONMLK"`, then each 3-letter segment is
  /// `["CAB", "XYZ", "PON", "MLK"]`. Each chunk is sorted individually:
  /// `["ABC", "XYZ", "NOP", "KLM"]`, then concatenated to produce `"ABCXYZNOPKLM"`.
  #[must_use]
  pub fn normalized(&self) -> String {
    self
      .input
      .chars()
      .collect::<Vec<_>>()
      .chunks(3)
      .flat_map(|chunk| {
        let mut sorted_chunk: Vec<char> = chunk.to_vec();
        sorted_chunk.sort_unstable();
        sorted_chunk.into_iter()
      })
      .collect()
  }
}

/// Validates a side (3-letter uppercase ASCII string).
///
/// Returns the validated string on success, or an error `String` describing what went wrong.
///
/// # Errors
///
/// * If the string is not ASCII.
/// * If the string is not exactly 3 letters long.
/// * If the letters are not all uppercase.
fn validate_side(side: &str) -> Result<&str, String> {
  if !side.is_ascii() {
    return Err(format!("The side '{side}' is not an ASCII string."));
  }
  if side.len() != 3 {
    return Err(format!(
      "The side '{side}' does not have exactly 3 letters."
    ));
  }
  if side.chars().all(|letter| !letter.is_ascii_uppercase()) {
    return Err(format!(
      "The letters of the side '{side}' are not all uppercase."
    ));
  }

  Ok(side)
}

impl TryFrom<&Value> for PuzzleInput {
  type Error = String;

  /// Constructs a [`PuzzleInput`] from a [`serde_json::Value`].
  ///
  /// # Expected Input
  ///
  /// This expects a JSON object with:
  ///
  /// * A `"sides"` array of exactly 4 string values, each three uppercase ASCII letters.
  /// * A `"printDate"` string in `YYYY-MM-DD` format.
  ///
  /// # Errors
  ///
  /// * If `"sides"` is missing or invalid.
  /// * If the array does not have exactly 4 entries.
  /// * If any of the sides is non-string or fails validation.
  /// * If `"printDate"` is missing or invalid, or if parsing fails.
  fn try_from(value: &Value) -> Result<Self, Self::Error> {
    // Extract the "sides" field as an array of strings
    let sides = value
      .get("sides")
      .and_then(|s| s.as_array())
      .ok_or("Missing or invalid 'sides' field")?;

    // Ensure we have exactly 4 sides
    if sides.len() != 4 {
      return Err("Expected 4 sides".into());
    }

    let [top, right, bottom, left] = [
      sides[0]
        .as_str()
        .ok_or(String::from("Non-string value found in 'sides' top value."))
        .and_then(validate_side)?,
      sides[1]
        .as_str()
        .ok_or(String::from(
          "Non-string value found in 'sides' right value.",
        ))
        .and_then(validate_side)?,
      sides[2]
        .as_str()
        .ok_or(String::from(
          "Non-string value found in 'sides' bottom value.",
        ))
        .and_then(validate_side)?,
      sides[3]
        .as_str()
        .ok_or(String::from(
          "Non-string value found in 'sides' left value.",
        ))
        .and_then(validate_side)?,
    ];

    // Combine the four sides into a 12-character input
    let input = format!("{top}{right}{bottom}{left}");

    // Extract the "printDate" field and parse it into a NaiveDate
    let print_date = value
      .get("printDate")
      .and_then(|d| d.as_str())
      .ok_or("Missing or invalid 'printDate' field")?;

    let date = NaiveDate::parse_from_str(print_date, "%Y-%m-%d")
      .map_err(|_| format!("Failed to parse printDate '{print_date}' as NaiveDate"))?;

    // Return the PuzzleInput struct
    Ok(PuzzleInput { date, input })
  }
}

impl InputsByDate {
  /// Inserts a [`PuzzleInput`] into the map, keyed by the reverse (descending) date.
  ///
  /// # Example
  ///
  /// ```
  /// let puzzle_input = PuzzleInput {
  ///   date: NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
  ///   input: "ABCDEFXYZUVW".to_string()
  /// };
  /// let mut inputs_by_date = InputsByDate::default();
  /// inputs_by_date.insert(&puzzle_input);
  /// ```
  pub fn insert(&mut self, puzzle_input: &PuzzleInput) {
    self
      .0
      .insert(Reverse(puzzle_input.date), puzzle_input.input.clone());
  }

  /// Reads [`InputsByDate`] from the file system, or creates a default, empty instance
  /// if the file does not exist or fails to parse.
  ///
  /// The file is expected to be named `inputsByDate.json` in the directory specified by `path`.
  ///
  /// # Example
  ///
  /// ```
  /// let inputs_by_date = InputsByDate::read_from_file_or_create(Path::new("./data"));
  /// ```
  #[must_use]
  pub fn read_from_file_or_create(path: &Path) -> Self {
    Self::read_from_file(path).unwrap_or_default()
  }

  /// Reads an [`InputsByDate`] from the `inputsByDate.json` file located in the given directory.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// * The file does not exist or cannot be opened.
  /// * JSON deserialization fails.
  pub fn read_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
    let path = path.join(INPUTS_BY_DATE);
    let file = File::open(&path)?;
    let inputs = serde_json::from_reader(file)?;

    Ok(inputs)
  }

  /// Writes this [`InputsByDate`] to `inputsByDate.json` in the given directory in pretty JSON format.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// * The file cannot be created.
  /// * JSON serialization fails.
  pub fn write_to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
    let path = path.join(INPUTS_BY_DATE);
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, self)?;
    Ok(())
  }
}

impl DatesByInput {
  /// Creates a new, empty `DatesByInput`.
  ///
  /// This method is identical to the `Default` implementation
  /// but is provided for convenience and clarity.
  #[must_use]
  pub fn new() -> Self {
    Self(BTreeMap::new())
  }

  /// Inserts a [`PuzzleInput`] into the map, keyed by the puzzle's normalized input string.
  /// The value stored is the puzzle’s date.
  ///
  /// # Example
  ///
  /// ```
  /// let puzzle_input = PuzzleInput {
  ///     date: NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
  ///     input: "CABXYZPONMLK".to_string()
  /// };
  /// let mut dates_by_input = DatesByInput::new();
  /// dates_by_input.insert(&puzzle_input);
  /// ```
  pub fn insert(&mut self, puzzle_input: &PuzzleInput) {
    self.0.insert(puzzle_input.normalized(), puzzle_input.date);
  }

  /// Reads [`DatesByInput`] from the file system, or creates a default (empty) instance
  /// if the file does not exist or fails to parse.
  ///
  /// The file is expected to be named `datesByInput.json` in the directory specified by `path`.
  ///
  /// # Example
  ///
  /// ```
  /// let dates_by_input = DatesByInput::read_from_file_or_create(Path::new("./data"));
  /// ```
  #[must_use]
  pub fn read_from_file_or_create(path: &Path) -> Self {
    Self::read_from_file(path).unwrap_or_default()
  }

  /// Reads a [`DatesByInput`] from the `datesByInput.json` file in the given directory.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// * The file does not exist or cannot be opened.
  /// * JSON deserialization fails.
  pub fn read_from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
    let path = path.join(DATES_BY_INPUT);
    let file = File::open(&path)?;
    let inputs = serde_json::from_reader(file)?;

    Ok(inputs)
  }

  /// Writes this [`DatesByInput`] to `datesByInput.json` in the given directory in pretty JSON format.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// * The file cannot be created.
  /// * JSON serialization fails.
  pub fn write_to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
    let path = path.join(DATES_BY_INPUT);
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, self)?;
    Ok(())
  }
}

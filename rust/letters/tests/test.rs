mod letter_group;
mod letter_sequence;
mod letter_set;
mod solution;

#[test]
fn compress_letter() {
  for (letter, expected) in (b'A'..=b'Z').zip(0..) {
    assert_eq!(
      expected,
      letters::compress_letter(letter),
      "A compressed letter should match the expected value.",
    );
  }
}

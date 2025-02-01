use letters::compress_letter;
use letters::create_letter_group_function;
use letters::LetterGroup;

#[test]
fn create_letter_group() {
  use LetterGroup::*;

  let letter_group = create_letter_group_function!("ABCDEFGHIJKL");

  let group1 = "ABC";
  let group2 = "DEF";
  let group3 = "GHI";
  let group4 = "JKL";
  let invalid = "XYZ";

  for letter in group1.as_bytes().iter().copied().map(compress_letter) {
    assert!(
      matches!(letter_group(letter), Group1),
      r#"A letter from "ABC" should be in {Group1:?}"#,
    );
  }

  for letter in group2.as_bytes().iter().copied().map(compress_letter) {
    assert!(
      matches!(letter_group(letter), Group2),
      r#"A letter from "ABC" should be in {Group2:?}"#,
    );
  }

  for letter in group3.as_bytes().iter().copied().map(compress_letter) {
    assert!(
      matches!(letter_group(letter), Group3),
      r#"A letter from "ABC" should be in {Group3:?}"#,
    );
  }

  for letter in group4.as_bytes().iter().copied().map(compress_letter) {
    assert!(
      matches!(letter_group(letter), Group4),
      r#"A letter from "ABC" should be in {Group4:?}"#,
    );
  }

  for letter in invalid.as_bytes().iter().copied().map(compress_letter) {
    assert!(
      matches!(letter_group(letter), Invalid),
      r#"A letter from "ABC" should be in {Invalid:?}"#,
    );
  }
}

#[test]
fn can_be_adjacent_to() {
  use LetterGroup::*;

  let letter_groups = [Invalid, Group1, Group2, Group3, Group4];

  for group in letter_groups {
    match group {
      Invalid => {
        for other in letter_groups {
          assert!(
            !group.can_be_adjacent_to(other),
            "{group:?} cannot be adjacent to {other:?}",
          );
        }
      }
      _ => {
        for other in letter_groups {
          match other {
            Invalid => assert!(
              !group.can_be_adjacent_to(other),
              "{group:?} cannot be adjacent to {other:?}",
            ),
            other => {
              if group == other {
                assert!(
                  !group.can_be_adjacent_to(other),
                  "{group:?} cannot be adjacent to {other:?}",
                );
              } else {
                assert!(
                  group.can_be_adjacent_to(other),
                  "{group:?} can be adjacent to {other:?}",
                );
              }
            }
          }
        }
      }
    }
  }
}

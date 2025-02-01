use letters::compress_letter;
use letters::LetterSequence;

#[test]
fn empty() {
  assert!(
    LetterSequence::empty().is_empty(),
    "An empty LetterSequence is empty."
  );

  assert_eq!(
    0,
    LetterSequence::empty().len(),
    "An empty LetterSequence has zero length."
  );
}

#[test]
fn is_empty() {
  assert!(
    LetterSequence::empty().is_empty(),
    "An empty LetterSequence is empty."
  );

  assert!(
    !LetterSequence::new("CONSIDERABLY").is_empty(),
    "A non-empty LetterSequence is not empty."
  );
}

#[test]
fn len() {
  assert_eq!(
    0,
    LetterSequence::empty().len(),
    "An empty LetterSequence has zero length.",
  );

  let letters = "ABCDEFGHIJKL";
  for n in 0..=letters.len() {
    assert_eq!(
      n,
      LetterSequence::new(&letters[0..n]).len(),
      "A LetterSequence with {n} letters has a length of {n}",
    );
  }
}

#[test]
fn has_all_letters() {
  assert!(
    !LetterSequence::empty().has_all_letters(),
    "An empty LetterSequence does not have all letters."
  );

  let letters = "ABCDEFGHIJKL";
  for n in 0..=letters.len() {
    let expected = n == 12;
    assert_eq!(
      expected,
      LetterSequence::new(&letters[0..n]).has_all_letters(),
      "A LetterSequence with {n} letters {}",
      if expected {
        "has all letters."
      } else {
        "does not have all letters."
      }
    );
  }
}

#[test]
fn with_letter() {
  assert!(
    !LetterSequence::empty().with_letter(b'A').is_empty(),
    "A LetterSequence with a letter is not empty."
  );

  let letters = "ABCDEFGHIJKL".as_bytes();
  let mut sequence = LetterSequence::empty();

  for (n, &letter) in letters.iter().enumerate() {
    let expected = n + 1;
    sequence = sequence.with_letter(letter);

    assert_eq!(
      expected,
      sequence.len(),
      "A LetterSequence with {n} letters has a length of {n}",
    );
  }
}

#[test]
fn new() {
  assert_eq!(
    LetterSequence::new(""),
    LetterSequence::empty(),
    "A LetterSequence created from an empty string is empty."
  );

  let letters = "ABCDEFGHIJKL";
  let bytes = letters.as_bytes();
  let mut sequence = LetterSequence::empty();

  for n in 0..letters.len() {
    sequence = sequence.with_letter(bytes[n]);
    assert_eq!(
      sequence,
      LetterSequence::new(&letters[0..=n]),
      "Two equal LetterSequences constructed with different methods differently should be equal."
    );
  }
}

#[test]
fn cut_from_start() {
  assert!(
    LetterSequence::empty()
      .with_letter(b'A')
      .cut_from_start(1)
      .is_empty(),
    "A LetterSequence with one letter is empty after cutting one letter form the start.",
  );

  let letters = "ABCDEFGHIJKL";
  let sequence = LetterSequence::new(letters);

  for n in 0..=12 {
    assert_eq!(
            sequence.cut_from_start(n),
            LetterSequence::new(&letters[n..]),
            "Cutting letters from the start of a LetterSequence is equal to slicing letters from the start of a str.",
        );
  }
}

#[test]
fn cut_from_end() {
  assert!(
    LetterSequence::empty()
      .with_letter(b'A')
      .cut_from_end(1)
      .is_empty(),
    "A LetterSequence with one letter is empty after cutting one letter form the end.",
  );

  let letters = "ABCDEFGHIJKL";
  let sequence = LetterSequence::new(letters);

  for n in 0..=12 {
    assert_eq!(
            sequence.cut_from_end(n),
            LetterSequence::new(&letters[0..(12 - n)]),
            "Cutting letters from the end of a LetterSequence is equal to slicing letters from the end of a str.",
        );
  }
}

#[test]
fn slice() {
  let letters = "ABCDEFGHIJKL";
  let sequence = LetterSequence::new(letters);

  assert!(
    sequence.slice(0..0).is_empty(),
    "An slice of no letters from a LetterSequence is empty.",
  );

  for start in 0..12 {
    for end in start..=12 {
      assert_eq!(
        sequence.slice(start..end),
        LetterSequence::new(&letters[start..end]),
        "A slice of a LetterSequence is equal to the slice of a str.",
      );
    }
  }
}

#[test]
fn letters() {
  let letters = "ABCDEFGHIJKL";
  let bytes = letters.as_bytes();
  let sequence = LetterSequence::new(letters);

  for start in 0..12 {
    for end in start..=12 {
      assert_eq!(
        sequence.slice(start..end).letters_rev().collect::<Vec<_>>(),
        bytes[start..end]
          .iter()
          .rev()
          .copied()
          .map(compress_letter)
          .collect::<Vec<_>>(),
        "The Letters iterator returns the same items as that from a slice of bytes."
      );
    }
  }
}

#[test]
fn ascii_bytes() {
  let letters = "ABCDEFGHIJKL";
  let bytes = letters.as_bytes();
  let sequence = LetterSequence::new(letters);

  for start in 0..12 {
    for end in start..=12 {
      assert_eq!(
        sequence.slice(start..end).ascii_bytes().collect::<Vec<_>>(),
        bytes[start..end].to_vec(),
        "The Letters iterator returns the same items as that from a slice of bytes."
      );
    }
  }
}

#[test]
fn can_append_to() {
  let prefix = LetterSequence::new("ABCDEFGHI");

  let valid_suffixes = [
    LetterSequence::new("IJK"),
    LetterSequence::new("IYZ"),
    LetterSequence::new("IJKL"),
  ];

  for suffix in valid_suffixes {
    assert!(
      suffix.can_append_to(prefix),
      r#"The suffix "{suffix}" can be appended to the prefix "{prefix}"."#,
    );
  }

  let invalid_suffixes_with_reasons = [
    (
      LetterSequence::from("JKE"),
      "does not join on the same letter.",
    ),
    (
      LetterSequence::from("IKJLM"),
      "extends the length beyond 12 letters.",
    ),
    (
      LetterSequence::from("IJE"),
      "shares more than one letter with the prefix.",
    ),
  ];

  for (suffix, reason) in invalid_suffixes_with_reasons {
    assert!(
      !suffix.can_append_to(prefix),
      r#"Cannot append the suffix "{suffix}" to the prefix "{prefix}" because it {reason}"#,
    );
  }
}

#[test]
fn can_prepend_to() {
  let prefix = LetterSequence::new("ABCDEFGHI");

  let valid_suffixes = [
    LetterSequence::new("IJK"),
    LetterSequence::new("IYZ"),
    LetterSequence::new("IJKL"),
  ];

  for suffix in valid_suffixes {
    assert!(
      prefix.can_prepend_to(suffix),
      r#"The prefix "{prefix}" can be prepended to the suffix "{suffix}"."#,
    );
  }

  let invalid_suffixes_with_reasons = [
    (
      LetterSequence::from("JKE"),
      "does not join on the same letter.",
    ),
    (
      LetterSequence::from("IKJLM"),
      "extends the length beyond 12 letters.",
    ),
    (
      LetterSequence::from("IJE"),
      "shares more than one letter with the suffix.",
    ),
  ];

  for (suffix, reason) in invalid_suffixes_with_reasons {
    assert!(
      !suffix.can_append_to(prefix),
      r#"Cannot prepend the prefix "{prefix}" to the suffix "{suffix}" because it {reason}"#,
    );
  }
}

#[test]
fn append_to() {
  assert_eq!(
    LetterSequence::new("FISHOME"),
    LetterSequence::new("HOME").append_to(LetterSequence::new("FISH")),
  );
  assert_eq!(
    LetterSequence::new("HOME").append_to(LetterSequence::new("FISH")),
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HOME")),
  );
  assert_eq!(
    LetterSequence::new("FISHAT"),
    LetterSequence::new("HAT").append_to(LetterSequence::new("FISH")),
  );
  assert_eq!(
    LetterSequence::new("HAT").append_to(LetterSequence::new("FISH")),
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HAT")),
  );
}

#[test]
fn prepend_to() {
  assert_eq!(
    LetterSequence::new("FISHOME"),
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HOME")),
  );
  assert_eq!(
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HOME")),
    LetterSequence::new("HOME").append_to(LetterSequence::new("FISH")),
  );
  assert_eq!(
    LetterSequence::new("FISHAT"),
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HAT")),
  );
  assert_eq!(
    LetterSequence::new("FISH").prepend_to(LetterSequence::new("HAT")),
    LetterSequence::new("HAT").append_to(LetterSequence::new("FISH")),
  );
}

#[test]
fn is_valid_word() {
  let letter_group = letters::create_letter_group_function!("ABCDEFGHIJKL");

  let group1 = "ABC";
  let group2 = "DEF";
  let group3 = "GHI";
  let group4 = "JKL";

  for &g1 in group1.as_bytes() {
    for &g2 in group2.as_bytes() {
      for &g3 in group3.as_bytes() {
        for &g4 in group4.as_bytes() {
          assert!(
            LetterSequence::new(&String::from_utf8(vec![g1, g2, g3, g4]).unwrap())
              .is_valid_word(&letter_group),
            "A LetterSequence formed with adjacent letters from each group is valid."
          );
        }
      }
    }
  }

  for &g1 in group1.as_bytes() {
    for &g1_invalid in group1.as_bytes().iter().filter(|&&byte| byte != g1) {
      for &g2 in group2.as_bytes() {
        for &g3 in group3.as_bytes() {
          for &g4 in group4.as_bytes() {
            assert!(
              !LetterSequence::new(&String::from_utf8(vec![g1, g1_invalid, g2, g3, g4]).unwrap())
                .is_valid_word(&letter_group),
              "A LetterSequence with two adjacent letters from group 1 is invalid.",
            );
          }
        }
      }
    }
  }

  for &g1 in group1.as_bytes() {
    for &g2 in group2.as_bytes() {
      for &g2_invalid in group2.as_bytes().iter().filter(|&&byte| byte != g2) {
        for &g3 in group3.as_bytes() {
          for &g4 in group4.as_bytes() {
            assert!(
              !LetterSequence::new(&String::from_utf8(vec![g1, g2, g2_invalid, g3, g4]).unwrap())
                .is_valid_word(&letter_group),
              "A LetterSequence with two adjacent letters from group 2 is invalid.",
            );
          }
        }
      }
    }
  }

  for &g1 in group1.as_bytes() {
    for &g2 in group2.as_bytes() {
      for &g3 in group3.as_bytes() {
        for &g3_invalid in group3.as_bytes().iter().filter(|&&byte| byte != g3) {
          for &g4 in group4.as_bytes() {
            assert!(
              !LetterSequence::new(&String::from_utf8(vec![g1, g2, g3, g3_invalid, g4]).unwrap())
                .is_valid_word(&letter_group),
              "A LetterSequence with two adjacent letters from group 3 is invalid.",
            );
          }
        }
      }
    }
  }

  for &g1 in group1.as_bytes() {
    for &g2 in group2.as_bytes() {
      for &g3 in group3.as_bytes() {
        for &g4 in group4.as_bytes() {
          for &g4_invalid in group4.as_bytes().iter().filter(|&&byte| byte != g4) {
            assert!(
              !LetterSequence::new(&String::from_utf8(vec![g1, g2, g3, g4, g4_invalid]).unwrap())
                .is_valid_word(&letter_group),
              "A LetterSequence with two adjacent letters from group 4 is invalid.",
            );
          }
        }
      }
    }
  }
}

use letters::compress_letter;
use letters::LetterSet;

#[test]
fn is_empty() {
  assert!(
    LetterSet::empty().is_empty(),
    "An empty LetterSet is empty."
  );
  assert!(
    !LetterSet::empty().insert(compress_letter(b'A')).is_empty(),
    "A non-empty LetterSet is not empty."
  );
}

#[test]
fn len() {
  let letter_set = LetterSet::empty();
  assert_eq!(
    0,
    letter_set.len(),
    "An empty LetterSet should have a length of zero."
  );

  let letter_set = letter_set.insert(compress_letter(b'A'));
  assert_eq!(
    1,
    letter_set.len(),
    "A LetterSet with one letter should have a length of one."
  );

  let letter_set = letter_set.insert(compress_letter(b'B'));
  assert_eq!(
    2,
    letter_set.len(),
    "A LetterSet with two letters should have a length of two."
  );

  let letter_set = letter_set
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));
  assert_eq!(
    6,
    letter_set.len(),
    "An LetterSet with six letters should have a length of six."
  );
}

#[test]
fn from_ascii_slice() {
  let letter_set = LetterSet::from_ascii_slice(b"ABC");

  assert_eq!(3, letter_set.len());
  assert!(letter_set.has(compress_letter(b'A')));
  assert!(letter_set.has(compress_letter(b'B')));
  assert!(letter_set.has(compress_letter(b'C')));
}

#[test]
fn has() {
  let empty_set = LetterSet::empty();

  for letter in compress_letter(b'A')..=compress_letter(b'Z') {
    assert!(
      !empty_set.has(letter),
      "An empty LetterSet should contain no letters."
    );
  }

  let letter_set = empty_set.insert(compress_letter(b'A'));
  assert!(
    letter_set.has(compress_letter(b'A')),
    "A LetterSet with 'a' should contain 'a'"
  );

  let letter_set = letter_set.insert(compress_letter(b'B'));
  assert!(
    letter_set.has(compress_letter(b'A')),
    "A LetterSet with ['a', 'b'] should contain 'a'"
  );
  assert!(
    letter_set.has(compress_letter(b'B')),
    "A LetterSet with ['a', 'b'] should contain 'b'"
  );

  let letter_set = letter_set.insert(compress_letter(b'Z'));
  assert!(
    letter_set.has(compress_letter(b'A')),
    "A LetterSet with ['a', 'b', 'z'] should contain 'a'"
  );
  assert!(
    letter_set.has(compress_letter(b'B')),
    "A LetterSet with ['a', 'b', 'z'] should contain 'b'"
  );
  assert!(
    letter_set.has(compress_letter(b'Z')),
    "A LetterSet with ['a', 'b', 'z'] should contain 'z'"
  );
}

#[test]
fn has_ascii() {
  let empty_set = LetterSet::empty();

  for letter in b'A'..=b'Z' {
    assert!(
      !empty_set.has_ascii(letter),
      "An empty LetterSet should contain no letters."
    );
  }

  let letter_set = empty_set.insert(compress_letter(b'A'));
  assert!(
    letter_set.has_ascii(b'A'),
    "A LetterSet with 'A' should contain 'A'"
  );

  let letter_set = letter_set.insert(compress_letter(b'B'));
  assert!(
    letter_set.has_ascii(b'A'),
    "A LetterSet with ['a', 'b'] should contain 'a'"
  );
  assert!(
    letter_set.has_ascii(b'B'),
    "A LetterSet with ['a', 'b'] should contain 'b'"
  );

  let letter_set = letter_set.insert(compress_letter(b'Z'));
  assert!(
    letter_set.has_ascii(b'A'),
    "A LetterSet with ['a', 'b', 'z'] should contain 'a'"
  );
  assert!(
    letter_set.has_ascii(b'B'),
    "A LetterSet with ['a', 'b', 'z'] should contain 'b'"
  );
  assert!(
    letter_set.has_ascii(b'Z'),
    "A LetterSet with ['a', 'b', 'z'] should contain 'z'"
  );
}

#[test]
fn insert() {
  let mut letter_set = LetterSet::empty();

  for (expected_length, letter) in (1..).zip(compress_letter(b'A')..=compress_letter(b'Z')) {
    letter_set = letter_set.insert(letter);
    assert_eq!(
      expected_length,
      letter_set.len(),
      "The LetterSet length should increase by 1 with each insertion.",
    );
  }

  for letter in compress_letter(b'A')..=compress_letter(b'Z') {
    assert!(
      letter_set.has(letter),
      "A full LetterSet should have every letter."
    );
  }
}

#[test]
fn intersection_with_empty_set() {
  assert!(
    LetterSet::empty()
      .intersection(LetterSet::empty())
      .is_empty(),
    "The intersection of two empty LetterSets should be an empty LetterSet."
  );

  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  assert!(
    fish_set.intersection(LetterSet::empty()).is_empty(),
    "The intersection of a LetterSet with an empty LetterSet should be an empty LetterSet",
  );
  assert!(
    LetterSet::empty().intersection(fish_set).is_empty(),
    "The intersection of an empty LetterSet with another LetterSet should be an empty LetterSet",
  );
}

#[test]
fn intersection_with_disjoint_sets() {
  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  let cat_set = LetterSet::empty()
    .insert(compress_letter(b'C'))
    .insert(compress_letter(b'A'))
    .insert(compress_letter(b'T'));

  assert_eq!(
    fish_set.intersection(cat_set),
    cat_set.intersection(fish_set),
    "The intersection of disjoint sets is commutative."
  );

  assert!(
    fish_set.intersection(cat_set).is_empty(),
    "The intersection of disjoint sets is the empty set.",
  );
}

#[test]
fn intersection_with_overlapping_sets() {
  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  let swim_set = LetterSet::empty()
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'W'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'M'));

  assert_eq!(
    fish_set.intersection(swim_set),
    swim_set.intersection(fish_set),
    "The intersection of overlapping sets is commutative."
  );

  let intersection = fish_set.intersection(swim_set);

  assert_eq!(
        intersection.len(),
        fish_set.len() + swim_set.len() - fish_set.union(swim_set).len(),
        "The length of an overlapping intersection is the sum of its subsets' lengths minus the length of their union",
    );

  for letter in "SI".as_bytes().iter().copied().map(compress_letter) {
    assert!(
      intersection.has(letter),
      "A intersection should contain only letters shared among its subsets.",
    );
  }
}

#[test]
fn union_with_empty_set() {
  assert!(
    LetterSet::empty()
      .intersection(LetterSet::empty())
      .is_empty(),
    "The union of two empty LetterSets should be an empty LetterSet."
  );

  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  assert_eq!(
    fish_set,
    fish_set.union(LetterSet::empty()),
    "The union of a LetterSet with an empty LetterSet should be the same LetterSet",
  );
  assert_eq!(
    fish_set,
    LetterSet::empty().union(fish_set),
    "The union of an empty LetterSet with another LetterSet should be the same LetterSet",
  );
}

#[test]
fn union_with_disjoint_sets() {
  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  let cat_set = LetterSet::empty()
    .insert(compress_letter(b'C'))
    .insert(compress_letter(b'A'))
    .insert(compress_letter(b'T'));

  assert_eq!(
    fish_set.union(cat_set),
    cat_set.union(fish_set),
    "The union of disjoint sets is commutative."
  );

  let union = fish_set.union(cat_set);

  assert_eq!(
    union.len(),
    fish_set.len() + cat_set.len(),
    "The length of a disjoint union is the sum of its subsets' lengths",
  );

  for letter in "CATFISH".as_bytes().iter().copied().map(compress_letter) {
    assert!(
      union.has(letter),
      "A union should contain every letter of its subsets.",
    );
  }
}

#[test]
fn union_with_overlapping_sets() {
  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  let swim_set = LetterSet::empty()
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'W'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'M'));

  assert_eq!(
    fish_set.union(swim_set),
    swim_set.union(fish_set),
    "The union of overlapping sets is commutative."
  );

  let union = fish_set.union(swim_set);

  assert_eq!(
        union.len(),
        fish_set.len() + swim_set.len() - fish_set.intersection(swim_set).len(),
        "The length of an overlapping union is the sum of its subsets' lengths minus the length of their intersection.",
    );

  for letter in "SWIMFISH".as_bytes().iter().copied().map(compress_letter) {
    assert!(
      union.has(letter),
      "A union should contain every letter of its subsets.",
    );
  }
}

#[test]
fn ascii_bytes() {
  let fish_set = LetterSet::empty()
    .insert(compress_letter(b'F'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'H'));

  let swim_set = LetterSet::empty()
    .insert(compress_letter(b'S'))
    .insert(compress_letter(b'W'))
    .insert(compress_letter(b'I'))
    .insert(compress_letter(b'M'));

  assert_eq!(
    "IS".as_bytes().to_vec(),
    fish_set
      .intersection(swim_set)
      .ascii_bytes()
      .collect::<Vec<_>>(),
  );

  assert_eq!(
    "FHIMSW".as_bytes().to_vec(),
    fish_set.union(swim_set).ascii_bytes().collect::<Vec<_>>(),
  );
}

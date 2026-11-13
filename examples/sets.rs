//! Properties with multiple values, and queries on what values look like.
//!
//! Run with `cargo run --example sets`.

use std::collections::BTreeSet;
use venndb::VennDB;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Skill {
    Rust,
    Design,
    Language { name: &'static str, native: bool },
}

#[derive(Debug, VennDB)]
pub struct Employee {
    name: String,
    #[venndb(filter, set)]
    skills: Vec<Skill>,
    // a function returning the values will do for a type that is not yours
    #[venndb(filter, members = BTreeSet::iter)]
    floors: BTreeSet<u8>,
    // the type of the values has to be told if the type of the property does not
    #[venndb(filter, members = weekdays, item = "char")]
    days: u8,
    #[venndb(filter)]
    country: String,
}

/// The days worked, kept as one bit per day.
fn weekdays(days: &u8) -> impl Iterator<Item = char> {
    let days = *days;
    "MTWTF"
        .chars()
        .enumerate()
        .filter(move |(day, _)| days >> day & 1 == 1)
        .map(|(_, c)| c)
}

fn names(result: Option<EmployeeDBQueryResult<'_>>) -> Vec<&str> {
    result
        .map(|result| result.iter().map(|e| e.name.as_str()).collect())
        .unwrap_or_default()
}

fn main() {
    let dutch = |native| Skill::Language {
        name: "dutch",
        native,
    };
    let employee = |name: &str, skills: &[Skill], floors: &[u8], days, country: &str| Employee {
        name: name.to_owned(),
        skills: skills.to_vec(),
        floors: floors.iter().copied().collect(),
        days,
        country: country.to_owned(),
    };
    let db = EmployeeDB::from_rows(vec![
        employee("Alice", &[Skill::Rust, dutch(true)], &[1, 2], 0b11111, "BE"),
        employee("Bob", &[Skill::Design, dutch(false)], &[2], 0b00001, "NL"),
        employee("Charlie", &[Skill::Rust], &[3], 0b00100, "BE"),
        employee("Dana", &[], &[], 0, "FR"),
    ]);

    // everyone with this skill, or with a skill for which the predicate,
    // called once per distinct skill and not once per row, returns true
    let mut query = db.query();
    query.skills(Skill::Rust);
    assert_eq!(names(query.execute()), ["Alice", "Charlie"]);
    let mut query = db.query();
    query.skills_where(|skill| matches!(skill, Skill::Language { native: true, .. }));
    assert_eq!(names(query.execute()), ["Alice"]);

    // values of one property are OR-ed, properties are AND-ed
    let mut query = db.query();
    query.skills(Skill::Rust).skills(Skill::Design).floors(2);
    assert_eq!(names(query.execute()), ["Alice", "Bob"]);

    let mut query = db.query();
    query.days('W');
    assert_eq!(names(query.execute()), ["Alice", "Charlie"]);

    // every "filter map" has such a method, not only sets
    let mut query = db.query();
    query.country_where(|country| country != "BE");
    assert_eq!(names(query.execute()), ["Bob", "Dana"]);
}

#[test]
fn runs() {
    main();
}

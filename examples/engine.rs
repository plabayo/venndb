//! The types a generated database is made of, used without the macro.
//!
//! Run with `cargo run --example engine`.

use venndb::{BoolColumn, Filter, KeyIndex, Plan, RowSet, RuntimeColumn, ValueColumn};

fn main() {
    let cities = ["ghent", "paris", "ghent", "tokyo", "ghent"];

    // a column per property: what is asked of it is up to you
    let mut by_name = KeyIndex::new();
    let mut is_capital = BoolColumn::new();
    let mut by_city = ValueColumn::new();
    for (row, city) in cities.into_iter().enumerate() {
        by_name.insert(format!("{city}-{row}"), row);
        is_capital.push(Some(city != "ghent"));
        by_city.push(row, &city);
    }
    assert_eq!(by_name.get("tokyo-3"), Some(3));

    // a set of rows is a list as long as it is small, a bitmask once it is not
    let even: RowSet = (0..cities.len()).filter(|row| row % 2 == 0).collect();
    let hidden = RuntimeColumn::new(cities.len());
    hidden.set(4, true);

    // a plan has a slot per filter that can be set more than once
    let mut plan = Plan::new(cities.len(), 2);
    is_capital.filter(&mut plan, 0, false);
    by_city.filter(&mut plan, 1, &"ghent", false);
    by_city.filter(&mut plan, 1, &"tokyo", false);
    plan.and(Filter::Is(&even));
    plan.and(Filter::Runtime(&hidden, false));

    let matches = plan.execute().expect("rows that match");
    assert!(matches.iter().eq([0, 2]));
    assert!([0, 2].contains(&matches.random()));
    assert_eq!(matches.random_where(|row| row > 0), Some(2));

    hidden.set(4, false);
    assert_eq!(plan.execute().unwrap().len(), 3);
}

#[test]
fn runs() {
    main();
}

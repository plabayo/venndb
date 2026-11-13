//! Rows that are refused, and how values of one property are combined.
//!
//! Run with `cargo run --example validator`.

use venndb::VennDB;

#[derive(Debug, VennDB)]
#[venndb(validator = my_validator_fn)]
pub struct Value {
    #[venndb(filter)]
    pub foo: String,
    pub bar: u32,
}

fn my_validator_fn(value: &Value) -> bool {
    !value.foo.is_empty() && value.bar > 0
}

fn main() {
    let mut db = ValueDB::default();
    assert!(
        db.append(Value {
            foo: "".to_owned(),
            bar: 42,
        })
        .is_err()
    ); // fails because foo == empty

    let db = ValueDB::from_iter([
        Value {
            foo: "a".to_owned(),
            bar: 8,
        },
        Value {
            foo: "b".to_owned(),
            bar: 12,
        },
        Value {
            foo: "c".to_owned(),
            bar: 16,
        },
    ])
    .unwrap();

    let mut query = db.query();
    query.foo("a".to_owned());
    query.foo("c".to_owned());
    let values: Vec<_> = query.execute().unwrap().iter().collect();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].bar, 8);
    assert_eq!(values[1].bar, 16);

    // the row that was refused comes back with the error
    let error = ValueDB::from_rows(vec![Value {
        foo: "d".to_owned(),
        bar: 0,
    }])
    .unwrap_err();
    assert_eq!(error.kind(), ValueDBErrorKind::InvalidRow);
    assert_eq!(error.into_input()[0].foo, "d");
}

#[test]
fn runs() {
    main();
}

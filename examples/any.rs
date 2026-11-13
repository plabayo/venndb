//! Rows that match any value asked for.
//!
//! Run with `cargo run --example any`.

use venndb::{Any, VennDB};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Department {
    Any,
    Hr,
    Engineering,
}

impl Any for Department {
    fn is_any(&self) -> bool {
        self == &Self::Any
    }
}

#[derive(Debug, VennDB)]
pub struct Employee {
    name: String,
    #[venndb(filter, any)]
    department: Department,
}

#[derive(Debug, VennDB)]
pub struct Value {
    // a function does as well, for a type that is not yours
    #[venndb(filter, any = is_any)]
    pub foo: String,
    pub bar: u32,
}

fn is_any(value: &String) -> bool {
    value == "*"
}

fn main() {
    let db = EmployeeDB::from_iter([
        Employee {
            name: "Jack".to_owned(),
            department: Department::Any,
        },
        Employee {
            name: "Derby".to_owned(),
            department: Department::Hr,
        },
    ]);
    let mut query = db.query();

    // will match Jack and Derby, as Jack is marked as Any, meaning it can work for w/e value
    let hr_employees: Vec<_> = query
        .department(Department::Hr)
        .execute()
        .unwrap()
        .iter()
        .collect();
    assert_eq!(hr_employees.len(), 2);

    // a value no row has still matches the rows that work for any value
    let engineers = query
        .reset()
        .department(Department::Engineering)
        .execute()
        .unwrap();
    assert_eq!(engineers.first().name, "Jack");

    let db = ValueDB::from_iter([
        Value {
            foo: "foo".to_owned(),
            bar: 8,
        },
        Value {
            foo: "*".to_owned(),
            bar: 16,
        },
    ]);

    let mut query = db.query();
    query.foo("*".to_owned());
    let value = query.execute().unwrap().any();
    // this will never match the row with bar == 8,
    // given foo != an any value
    assert_eq!(value.bar, 16);
}

#[test]
fn runs() {
    main();
}

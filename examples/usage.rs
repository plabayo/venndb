//! The quick introduction of the README.
//!
//! Run with `cargo run --example usage`.

use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: Option<bool>,
    is_admin: bool,
    #[venndb(skip)]
    foo: bool,
    #[venndb(filter, any)]
    department: Department,
    #[venndb(filter)]
    country: Option<String>,
}

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

fn employees() -> Vec<Employee> {
    let employee = |id, name: &str, is_manager, is_admin, department, country: &str| Employee {
        id,
        name: name.to_owned(),
        is_manager,
        is_admin,
        foo: false,
        department,
        country: (!country.is_empty()).then(|| country.to_owned()),
    };
    vec![
        employee(1, "Alice", Some(true), true, Department::Engineering, "BE"),
        employee(2, "Bob", Some(false), true, Department::Engineering, "BE"),
        employee(3, "Charlie", Some(false), false, Department::Hr, "NL"),
        employee(4, "Dana", None, true, Department::Any, ""),
    ]
}

fn main() {
    let db = EmployeeDB::from_iter(employees()).expect("unique ids");

    let mut query = db.query();
    let employee = query
        .is_admin(true)
        .is_manager(false)
        .department(Department::Engineering)
        .execute()
        .expect("to have found at least one")
        .any();

    println!("non-manager admin engineer: {:?}", employee);
    assert_eq!(employee.name, "Bob");

    // keys look a row up directly
    assert_eq!(db.get_by_id(&3).unwrap().name, "Charlie");
    // a filter map only matches the rows that have a value for it
    assert_eq!(
        query
            .reset()
            .country("BE".to_owned())
            .execute()
            .unwrap()
            .len(),
        2
    );
    // a property that is skipped is no filter
    assert!(!employee.foo);
}

#[test]
fn runs() {
    main();
}

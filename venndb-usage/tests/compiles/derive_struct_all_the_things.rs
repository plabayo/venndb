use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
#[venndb(name = "EmployeeSheet", validator = employee_validator)]
struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    #[venndb(filter)] // explicit bool filter == regular bool
    is_manager: bool,
    is_admin: bool,
    is_something: Option<bool>,
    #[venndb(skip)]
    is_active: bool,
    #[venndb(filter, any)]
    department: Department,
    #[venndb(filter)]
    country: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Department {
    Any,
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn employee_validator(employee: &Employee) -> bool {
    employee.id > 0 && !employee.name.is_empty()
}

impl Any for Department {
    fn is_any(&self) -> bool {
        self == &Department::Any
    }
}

fn main() {
    let mut db = EmployeeSheet::new();
    db.append(Employee {
        id: 1,
        name: "Alice".to_string(),
        is_manager: true,
        is_admin: false,
        is_something: None,
        is_active: true,
        department: Department::Engineering,
        country: None,
    })
    .unwrap();

    let employee_ref = db.get_by_id(&1).unwrap();
    assert_eq!(employee_ref.id, 1);
    assert_eq!(employee_ref.name, "Alice");
    assert_eq!(employee_ref.is_something, None);
    assert_eq!(employee_ref.country, None);

    let mut query = db.query();
    query
        .is_manager(true)
        .is_admin(true)
        .department(Department::Engineering);
    assert!(query.execute().is_none());
}

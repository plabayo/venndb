use venndb::VennDB;

#[derive(Debug, VennDB)]
#[venndb(name = "EmployeeSheet")]
struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    #[venndb(filter)] // explicit bool filter == regular bool
    is_manager: bool,
    is_admin: bool,
    #[venndb(skip)]
    is_active: bool,
    #[venndb(filter)]
    department: Department,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {
    let mut db = EmployeeSheet::new();
    db.append(Employee {
        id: 1,
        name: "Alice".to_string(),
        is_manager: true,
        is_admin: false,
        is_active: true,
        department: Department::Engineering,
    })
    .unwrap();

    let employee_ref = db.get_by_id(&1).unwrap();
    assert_eq!(employee_ref.id, 1);
    assert_eq!(employee_ref.name, "Alice");

    let mut query = db.query();
    query.is_manager(true).is_admin(true);
    assert!(query.execute().is_none());
}

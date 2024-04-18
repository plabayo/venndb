use venndb::VennDB;

#[derive(Debug, VennDB)]
#[venndb(validator = "employee_validator")]
struct Employee {
    pub id: u32,
    pub name: String,
    pub is_manager: bool,
    pub is_admin: bool,
    pub is_active: bool,
    pub department: Department,
}

#[derive(Debug)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn employee_validator(employee: &Employee) -> bool {
    employee.id > 0 && !employee.name.is_empty()
}

fn main() {
    let _ = EmployeeDB::new();
}

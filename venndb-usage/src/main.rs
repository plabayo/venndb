#![allow(dead_code)]

use venndb::VennDB;

#[derive(Debug, VennDB)]
pub struct Employee {
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
    department: Department,
}

#[derive(Debug, PartialEq)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

fn main() {
    let employee = Employee {
        id: 1,
        name: "Alice".to_string(),
        is_manager: true,
        is_admin: false,
        is_active: true,
        department: Department::Engineering,
    };
    println!("employee: {:#?}", employee);

    let _db = EmployeeDB::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee() {
        let employee = Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        };
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");
        assert!(employee.is_manager);
        assert!(!employee.is_admin);
        assert!(employee.is_active);
        assert_eq!(employee.department, Department::Engineering);
    }

    #[test]
    fn test_employee_db_empty_len() {
        let db = EmployeeDB::new();
        assert_eq!(db.len(), 0);
    }
}

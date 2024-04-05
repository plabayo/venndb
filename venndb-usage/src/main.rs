#![allow(dead_code)]

use venndb::VennDB;

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
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
    fn test_employee_db_empty() {
        let db = EmployeeDB::new();
        assert_eq!(db.len(), 0);
        assert_eq!(db.capacity(), 0);
    }

    #[test]
    fn test_employee_db_append() {
        let mut db = EmployeeDB::default();
        assert_eq!(db.len(), 0);
        assert_eq!(db.capacity(), 0);

        let employee = Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        };

        db.append(employee);
        assert_eq!(db.len(), 1);

        assert!(db.get_by_id(&0).is_none());

        let employee: &Employee = db.get_by_id(&1).unwrap();
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");
    }
}

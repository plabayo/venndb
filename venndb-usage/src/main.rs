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

    #[test]
    fn test_employee_query_filters() {
        let mut db = EmployeeDB::default();

        db.append(Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        });

        let mut query = db.query();
        let results: Vec<_> = query
            .is_manager(true)
            .is_admin(true)
            .execute()
            .unwrap()
            .iter()
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 3);
        assert_eq!(query.execute().unwrap().first().id, 3);

        let mut query = db.query();
        assert!(query.is_active(false).execute().is_none());
    }

    #[test]
    #[should_panic]
    fn test_employee_duplicate_key() {
        // TODO: replace with error instead of panic
        let mut db = EmployeeDB::default();
        db.append(Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });

        // duplicate key: id (=1)
        db.append(Employee {
            id: 1,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
    }

    // TODO: add test to ensure that no other keys
    // have already been inserted!

    #[test]
    fn test_into_from_rows() {
        let rows = vec![
            Employee {
                id: 1,
                name: "Alice".to_string(),
                is_manager: true,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            },
            Employee {
                id: 2,
                name: "Bob".to_string(),
                is_manager: false,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            },
        ];

        let db = EmployeeDB::from_rows(rows);

        assert_eq!(db.len(), 2);
        assert_eq!(db.capacity(), 2);

        let mut query = db.query();
        query.is_manager(true);
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);

        let rows = db.into_rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[1].id, 2);
    }

    #[test]
    fn test_query_reset() {
        let mut db = EmployeeDB::default();

        db.append(Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        });

        let mut query = db.query();
        query.is_manager(true);
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 3);

        query.reset();
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
        assert_eq!(results[2].id, 3);
    }

    #[test]
    fn test_query_result_any() {
        let mut db = EmployeeDB::default();

        db.append(Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        });
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        });

        let mut query = db.query();
        query.is_active(true);
        let result = query.execute().unwrap().any();
        assert!(result.id == 1 || result.id == 2 || result.id == 3);
    }
}

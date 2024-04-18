#![allow(dead_code)]

use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    #[venndb(key)]
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
    #[venndb(filter, any)]
    department: Department,
}

#[derive(Debug)]
pub struct L1Engineer {
    id: u32,
    name: String,
}

impl From<L1Engineer> for Employee {
    fn from(engineer: L1Engineer) -> Employee {
        Employee {
            id: engineer.id,
            name: engineer.name,
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Department {
    Any,
    Engineering,
    Sales,
    Marketing,
    HR,
}

impl Any for Department {
    fn is_any(&self) -> bool {
        self == &Department::Any
    }
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

        db.append(employee).unwrap();
        assert_eq!(db.len(), 1);

        assert!(db.get_by_id(&0).is_none());

        let employee: &Employee = db.get_by_id(&1).unwrap();
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");

        let employee: &Employee = db.get_by_name("Alice").unwrap();
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");
    }

    #[test]
    fn test_append_into() {
        let mut db = EmployeeDB::default();
        db.append(L1Engineer {
            id: 1,
            name: "Alice".to_string(),
        })
        .unwrap();
        assert_eq!(db.len(), 1);

        let employee: &Employee = db.get_by_id(&1).unwrap();
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");
        assert!(!employee.is_manager);
        assert!(!employee.is_admin);
        assert!(employee.is_active);
        assert_eq!(employee.department, Department::Engineering);
    }

    #[test]
    fn test_extend() {
        let mut db = EmployeeDB::default();
        assert_eq!(db.len(), 0);
        assert!(db.get_by_id(&1).is_none());
        assert!(db.get_by_id(&2).is_none());
        assert!(db.is_empty());

        db.extend(vec![
            L1Engineer {
                id: 1,
                name: "Alice".to_string(),
            },
            L1Engineer {
                id: 2,
                name: "Bob".to_string(),
            },
        ])
        .unwrap();
        assert_eq!(db.len(), 2);

        let employee: &Employee = db.get_by_id(&1).unwrap();
        assert_eq!(employee.id, 1);
        assert_eq!(employee.name, "Alice");

        let employee: &Employee = db.get_by_id(&2).unwrap();
        assert_eq!(employee.id, 2);
        assert_eq!(employee.name, "Bob");
    }

    #[test]
    fn test_extend_duplicate_key() {
        let mut db = EmployeeDB::default();
        db.extend(vec![
            L1Engineer {
                id: 1,
                name: "Alice".to_string(),
            },
            L1Engineer {
                id: 2,
                name: "Bob".to_string(),
            },
        ])
        .unwrap();
        assert_eq!(db.len(), 2);

        let err = db
            .extend(vec![
                L1Engineer {
                    id: 2,
                    name: "Charlie".to_string(),
                },
                L1Engineer {
                    id: 3,
                    name: "David".to_string(),
                },
            ])
            .unwrap_err();
        assert_eq!(EmployeeDBErrorKind::DuplicateKey, err.kind());

        let (dup_employee, employee_iter) = err.into_input();
        assert_eq!(dup_employee.id, 2);
        assert_eq!(dup_employee.name, "Charlie");

        let employees: Vec<_> = employee_iter.collect();
        assert_eq!(employees.len(), 1);
        assert_eq!(employees[0].id, 3);

        db.extend(employees).unwrap();
        assert_eq!(db.len(), 3);
        assert_eq!(db.get_by_id(&3).unwrap().name, "David");
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
        })
        .unwrap();
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        })
        .unwrap();
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        })
        .unwrap();

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
    fn test_employee_duplicate_key() {
        let mut db = EmployeeDB::default();
        db.append(Employee {
            id: 1,
            name: "Alice".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        })
        .unwrap();

        // duplicate key: id (=1)
        let err = db
            .append(Employee {
                id: 1,
                name: "Bob".to_string(),
                is_manager: false,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            })
            .unwrap_err();
        assert_eq!(EmployeeDBErrorKind::DuplicateKey, err.kind());
        assert_eq!("Bob", err.into_input().name);
    }

    #[test]
    fn test_duplicate_key_with_zero_pollution() {
        #[derive(Debug, VennDB)]
        struct MultiKey {
            #[venndb(key)]
            a: String,
            #[venndb(key)]
            b: String,
            #[venndb(key)]
            c: String,
            d: bool,
            e: bool,
        }

        let mut db = MultiKeyDB::from_rows(vec![
            MultiKey {
                a: "a".to_string(),
                b: "b".to_string(),
                c: "c".to_string(),
                d: true,
                e: false,
            },
            MultiKey {
                a: "A".to_string(),
                b: "B".to_string(),
                c: "C".to_string(),
                d: false,
                e: true,
            },
        ])
        .unwrap();

        let err = db
            .append(MultiKey {
                a: "foo".to_string(),
                b: "bar".to_string(),
                c: "c".to_string(),
                d: false,
                e: true,
            })
            .unwrap_err();
        assert_eq!(MultiKeyDBErrorKind::DuplicateKey, err.kind());

        // ensure there was no pollution,
        // this will panic in ase there was
        assert!(db.get_by_a("foo").is_none());
        assert!(db.get_by_b("bar").is_none());
    }

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

        let db = EmployeeDB::from_rows(rows).unwrap();

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
    fn test_from_rows_duplicate_key() {
        let err = EmployeeDB::from_rows(vec![
            Employee {
                id: 1,
                name: "Alice".to_string(),
                is_manager: true,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            },
            Employee {
                id: 1,
                name: "Bob".to_string(),
                is_manager: false,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            },
        ])
        .unwrap_err();
        assert_eq!(EmployeeDBErrorKind::DuplicateKey, err.kind());

        let employees = err.into_input();
        assert_eq!(employees.len(), 2);
        assert_eq!(employees[0].name, "Alice");
        assert_eq!(employees[1].name, "Bob");
    }

    #[test]
    fn test_from_iter() {
        let db = EmployeeDB::from_iter([
            L1Engineer {
                id: 1,
                name: "Alice".to_string(),
            },
            L1Engineer {
                id: 2,
                name: "Bob".to_string(),
            },
        ])
        .unwrap();

        assert_eq!(db.len(), 2);
        assert_eq!(db.capacity(), 2);

        let mut query = db.query();
        query.is_manager(true);
        assert!(query.execute().is_none());

        query
            .reset()
            .is_manager(false)
            .department(Department::Engineering);
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_from_iter_duplicate_key() {
        let err = EmployeeDB::from_iter([
            L1Engineer {
                id: 1,
                name: "Alice".to_string(),
            },
            L1Engineer {
                id: 1,
                name: "Bob".to_string(),
            },
        ])
        .unwrap_err();
        assert_eq!(EmployeeDBErrorKind::DuplicateKey, err.kind());

        let employees = err.into_input();
        assert_eq!(employees.len(), 2);
        assert_eq!(employees[0].name, "Alice");
        assert_eq!(employees[1].name, "Bob");
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
        })
        .unwrap();
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        })
        .unwrap();
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        })
        .unwrap();

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
        })
        .unwrap();
        db.append(Employee {
            id: 2,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: true,
            department: Department::Engineering,
        })
        .unwrap();
        db.append(Employee {
            id: 3,
            name: "Charlie".to_string(),
            is_manager: true,
            is_admin: true,
            is_active: true,
            department: Department::Sales,
        })
        .unwrap();

        let mut query = db.query();
        query.is_active(true);
        let result = query.execute().unwrap().any();
        assert!(result.id == 1 || result.id == 2 || result.id == 3);
    }

    #[test]
    fn test_db_without_keys() {
        #[derive(Debug, VennDB)]
        struct NoKeys {
            name: String,
            a: bool,
            b: bool,
        }

        let mut db = NoKeysDB::from_rows(vec![
            NoKeys {
                name: "Alice".to_string(),
                a: true,
                b: false,
            },
            NoKeys {
                name: "Bob".to_string(),
                a: false,
                b: true,
            },
        ]);

        assert_eq!(db.len(), 2);
        assert_eq!(db.capacity(), 2);

        let mut query = db.query();
        query.a(true);
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Alice");

        db.append(NoKeys {
            name: "Charlie".to_string(),
            a: true,
            b: true,
        });

        let mut query = db.query();
        query.b(true);
        let results: Vec<_> = query.execute().unwrap().iter().collect();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Bob");
        assert_eq!(results[1].name, "Charlie");
    }

    #[test]
    fn test_db_iter() {
        let db = EmployeeDB::from_rows(vec![
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
        ])
        .unwrap();

        let mut iter = db.iter();
        assert_eq!(iter.next().unwrap().id, 1);
        assert_eq!(iter.next().unwrap().id, 2);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_db_result_filter() {
        let db = EmployeeDB::from_rows(vec![
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
        ])
        .unwrap();

        let mut query = db.query();
        query.is_active(true);
        let results = query.execute().unwrap();
        let rows = results.iter().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[1].id, 2);

        let results = results
            .filter(|r| r.department == Department::Engineering)
            .unwrap();
        let rows = results.iter().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[1].id, 2);

        let results = results.filter(|r| r.is_manager).unwrap();
        let rows = results.iter().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 1);

        assert!(results.filter(|r| r.is_admin).is_none());
    }

    #[test]
    fn test_db_filter_map() {
        let db = EmployeeDB::from_rows(vec![
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
            Employee {
                id: 3,
                name: "Charlie".to_string(),
                is_manager: true,
                is_admin: true,
                is_active: true,
                department: Department::Sales,
            },
            Employee {
                id: 4,
                name: "David".to_string(),
                is_manager: false,
                is_admin: true,
                is_active: true,
                department: Department::HR,
            },
            Employee {
                id: 5,
                name: "Eve".to_string(),
                is_manager: true,
                is_admin: false,
                is_active: true,
                department: Department::HR,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Marketing);
        assert!(query.execute().is_none());

        query.reset().department(Department::Engineering);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);

        query.reset().department(Department::HR);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 4);
        assert_eq!(results[1].id, 5);

        query.reset().department(Department::Sales);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 3);

        // all the filters
        let results = query
            .reset()
            .department(Department::Engineering)
            .is_manager(true)
            .is_admin(false)
            .is_active(true)
            .execute()
            .unwrap()
            .iter()
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }
}

#[cfg(test)]
mod tests_v0_2 {
    use super::*;

    #[derive(Debug, VennDB)]
    pub struct Worker {
        #[venndb(key)]
        id: u32,
        is_admin: bool,
        is_active: Option<bool>,
        #[venndb(filter)]
        department: Option<Department>,
    }

    #[test]
    fn test_optional_bool_filter() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: None,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.is_active(true);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_optional_map_filter() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: None,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_db_with_optional_properties_default_filter() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: None,
            },
        ])
        .unwrap();

        let query = db.query();
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_optional_map_filter_specific() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: None,
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: Some(Department::HR),
            },
            Worker {
                id: 3,
                is_admin: false,
                is_active: None,
                department: None,
            },
            Worker {
                id: 4,
                is_admin: false,
                is_active: None,
                department: Some(Department::Engineering),
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 4);
    }
}

#[cfg(test)]
mod tests_v0_2_1 {
    use super::*;

    #[derive(Debug, VennDB)]
    pub struct Worker {
        #[venndb(key)]
        id: u32,
        is_admin: bool,
        is_active: Option<bool>,
        #[venndb(filter, any)]
        department: Option<Department>,
    }

    #[test]
    fn test_any_filter_map() {
        let db = EmployeeDB::from_rows(vec![
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
                department: Department::HR,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Any);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_any_option_filter_map() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::HR),
            },
            Worker {
                id: 3,
                is_admin: false,
                is_active: None,
                department: None,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Any);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
        assert_eq!(results[2].id, 3);
    }

    #[test]
    fn test_any_row_filter_map() {
        let db = EmployeeDB::from_rows(vec![
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
                department: Department::Any,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_any_row_optional_filter_map() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: None,
            },
            Worker {
                id: 3,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Any),
            },
            Worker {
                id: 4,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::HR),
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 3);
    }
}

#[cfg(test)]
mod tests_v0_2_2 {
    use super::*;

    #[derive(Debug, VennDB)]
    pub struct Worker {
        #[venndb(key)]
        id: u32,
        is_admin: bool,
        is_active: Option<bool>,
        #[venndb(filter, any)]
        department: Option<Department>,
    }

    // regression test: <https://github.com/plabayo/venndb/issues/5>
    #[test]
    fn test_any_row_optional_filter_map_white_rabbit() {
        let db = WorkerDB::from_rows(vec![
            Worker {
                id: 1,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Engineering),
            },
            Worker {
                id: 2,
                is_admin: false,
                is_active: None,
                department: None,
            },
            Worker {
                id: 3,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::Any),
            },
            Worker {
                id: 4,
                is_admin: false,
                is_active: Some(true),
                department: Some(Department::HR),
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Marketing);
        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 3);
    }
}

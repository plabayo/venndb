#![allow(dead_code)]

use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
#[venndb(validator = employee_validator)]
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

fn employee_validator(employee: &Employee) -> bool {
    employee.id > 0
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
mod tests_v0_1 {
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

    // these two tests are no longer correct since
    // the fix introduced in issue https://github.com/plabayo/venndb/issues/7
    //
    // this is intended. As such these issues have moved to `::tests_v0_4`.
    // Check out the above issue if you want to find the motivation why.

    // #[test]
    // fn test_any_filter_map() {
    //     let db = EmployeeDB::from_rows(vec![
    //         Employee {
    //             id: 1,
    //             name: "Alice".to_string(),
    //             is_manager: true,
    //             is_admin: false,
    //             is_active: true,
    //             department: Department::Engineering,
    //         },
    //         Employee {
    //             id: 2,
    //             name: "Bob".to_string(),
    //             is_manager: false,
    //             is_admin: false,
    //             is_active: true,
    //             department: Department::HR,
    //         },
    //     ])
    //     .unwrap();

    //     let mut query = db.query();
    //     query.department(Department::Any);
    //     let results = query.execute().unwrap().iter().collect::<Vec<_>>();
    //     assert_eq!(results.len(), 2);
    //     assert_eq!(results[0].id, 1);
    //     assert_eq!(results[1].id, 2);
    // }

    // #[test]
    // fn test_any_option_filter_map() {
    //     let db = WorkerDB::from_rows(vec![
    //         Worker {
    //             id: 1,
    //             is_admin: false,
    //             is_active: Some(true),
    //             department: Some(Department::Engineering),
    //         },
    //         Worker {
    //             id: 2,
    //             is_admin: false,
    //             is_active: Some(true),
    //             department: Some(Department::HR),
    //         },
    //         Worker {
    //             id: 3,
    //             is_admin: false,
    //             is_active: None,
    //             department: None,
    //         },
    //     ])
    //     .unwrap();

    //     let mut query = db.query();
    //     query.department(Department::Any);
    //     let results = query.execute().unwrap().iter().collect::<Vec<_>>();
    //     assert_eq!(results.len(), 3);
    //     assert_eq!(results[0].id, 1);
    //     assert_eq!(results[1].id, 2);
    //     assert_eq!(results[2].id, 3);
    // }

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
mod tests_v0_3_0 {
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

#[cfg(test)]
mod tests_v0_4 {
    use super::*;

    #[derive(Debug, VennDB)]
    #[venndb(validator = worker_validator)]
    pub struct Worker {
        #[venndb(key)]
        id: u32,
        is_admin: bool,
        is_active: Option<bool>,
        #[venndb(filter, any)]
        department: Option<Department>,
    }

    fn worker_validator(worker: &Worker) -> bool {
        worker.id > 0 && (worker.is_active.unwrap_or_default() || !worker.is_admin)
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

        // no row matches the filter,
        // given all rows have an explicit department value
        assert!(query.execute().is_none());
    }

    #[test]
    fn test_any_filter_map_match() {
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
        query.department(Department::Any);

        let employee = query.execute().unwrap().any();
        assert_eq!(employee.id, 2);
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

        // no row matches the filter,
        // given all rows have an explicit department value
        assert!(query.execute().is_none());
    }

    #[test]
    fn test_any_option_filter_map_match() {
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
                department: Some(Department::Any),
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

        let employee = query.execute().unwrap().any();
        assert_eq!(employee.id, 2);
    }

    #[test]
    fn test_worker_db_valid_rows_append() {
        let mut db = WorkerDB::default();

        db.append(Worker {
            id: 1,
            is_admin: false,
            is_active: Some(true),
            department: Some(Department::Engineering),
        })
        .unwrap();

        db.append(Worker {
            id: 2,
            is_admin: false,
            is_active: None,
            department: None,
        })
        .unwrap();

        db.append(Worker {
            id: 3,
            is_admin: false,
            is_active: Some(true),
            department: Some(Department::Any),
        })
        .unwrap();

        db.append(Worker {
            id: 4,
            is_admin: true,
            is_active: Some(true),
            department: Some(Department::HR),
        })
        .unwrap();
    }

    #[test]
    fn test_worker_db_valid_rows_from_iter() {
        WorkerDB::from_iter([
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
                is_admin: true,
                is_active: Some(true),
                department: Some(Department::HR),
            },
        ])
        .unwrap();
    }

    #[test]
    fn test_worker_db_invalid_rows_append() {
        let mut db = WorkerDB::default();

        assert_eq!(
            WorkerDBErrorKind::InvalidRow,
            db.append(Worker {
                id: 0,
                is_admin: false,
                is_active: None,
                department: Some(Department::Engineering),
            })
            .unwrap_err()
            .kind()
        );

        assert_eq!(
            WorkerDBErrorKind::InvalidRow,
            db.append(Worker {
                id: 1,
                is_admin: true,
                is_active: Some(false),
                department: Some(Department::Engineering),
            })
            .unwrap_err()
            .kind()
        );

        assert_eq!(
            WorkerDBErrorKind::InvalidRow,
            db.append(Worker {
                id: 2,
                is_admin: true,
                is_active: None,
                department: Some(Department::Engineering),
            })
            .unwrap_err()
            .kind()
        );
    }

    #[test]
    fn test_worker_db_invalid_rows_from_iter() {
        assert_eq!(
            WorkerDBErrorKind::InvalidRow,
            WorkerDB::from_iter(
                [Worker {
                    id: 0,
                    is_admin: false,
                    is_active: None,
                    department: Some(Department::Engineering),
                },]
                .into_iter(),
            )
            .unwrap_err()
            .kind()
        );
    }

    #[test]
    fn test_employee_db_append_invalid_row() {
        let mut db = EmployeeDB::default();

        assert_eq!(
            EmployeeDBErrorKind::InvalidRow,
            db.append(Employee {
                id: 0,
                name: "Alice".to_string(),
                is_manager: true,
                is_admin: false,
                is_active: true,
                department: Department::Engineering,
            })
            .unwrap_err()
            .kind()
        );
    }
}

#[cfg(test)]
mod tests_v0_5 {
    use super::*;

    #[test]
    fn test_employee_db_multi_filter_map_value() {
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
            Employee {
                id: 3,
                name: "Charlie".to_string(),
                is_manager: true,
                is_admin: true,
                is_active: true,
                department: Department::Sales,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        query.department(Department::HR);

        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 2);
    }

    #[test]
    fn test_employee_db_multi_filter_map_value_with_any() {
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
            Employee {
                id: 3,
                name: "Charlie".to_string(),
                is_manager: true,
                is_admin: true,
                is_active: true,
                department: Department::Any,
            },
        ])
        .unwrap();

        let mut query = db.query();
        query.department(Department::Engineering);
        query.department(Department::Any);

        let results = query.execute().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 1);
        assert_eq!(results[1].id, 3);
    }
}

#[cfg(test)]
mod tests_v0_7 {
    use std::collections::{BTreeSet, HashSet};
    use venndb::{Filter, FilterSet, VennDB};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Skill {
        Rust { years: u8 },
        Go { years: u8 },
        Design,
    }

    /// A set type of our own.
    #[derive(Debug, Default)]
    pub struct Tags(BTreeSet<&'static str>);

    impl FilterSet for Tags {
        type Item = &'static str;

        fn for_each_member(&self, f: impl FnMut(&&'static str)) {
            self.0.iter().for_each(f);
        }
    }

    fn is_everywhere(office: &String) -> bool {
        office == "*"
    }

    fn badge(engineer: &Engineer) -> String {
        format!("{}-{}", engineer.team, engineer.id % 8)
    }

    fn languages(spoken: &u8) -> impl Iterator<Item = char> {
        let spoken = *spoken;
        ['e', 'f', 'd']
            .into_iter()
            .enumerate()
            .filter(move |(bit, _)| spoken >> bit & 1 == 1)
            .map(|(_, language)| language)
    }

    #[derive(Debug, VennDB)]
    pub struct Engineer {
        #[venndb(key)]
        id: u32,
        #[venndb(key = badge)]
        badge: String,
        team: &'static str,
        remote: bool,
        #[venndb(filter, set)]
        skills: BTreeSet<Skill>,
        #[venndb(filter, set)]
        tags: Tags,
        #[venndb(filter, set)]
        floors: Option<BTreeSet<u8>>,
        #[venndb(filter, members = HashSet::iter)]
        projects: HashSet<u16>,
        #[venndb(filter, members = languages, item = char)]
        languages: u8,
        #[venndb(filter, any = is_everywhere)]
        office: String,
        #[venndb(filter, any = is_everywhere)]
        desk: Option<String>,
    }

    fn engineer(id: u32, team: &'static str, skills: &[Skill], office: &str) -> Engineer {
        Engineer {
            id,
            badge: String::new(),
            team,
            remote: id.is_multiple_of(2),
            skills: skills.iter().cloned().collect(),
            tags: Tags(if id < 3 {
                ["early", team].into()
            } else {
                BTreeSet::new()
            }),
            floors: (id != 2).then(|| [id as u8, 9].into()),
            projects: [id as u16 * 10, 7].into(),
            languages: id as u8,
            office: office.to_owned(),
            desk: (id != 4).then(|| if id == 1 { "*" } else { "a1" }.to_owned()),
        }
    }

    fn db() -> EngineerDB {
        EngineerDB::from_rows(vec![
            engineer(
                1,
                "core",
                &[Skill::Rust { years: 5 }, Skill::Design],
                "ghent",
            ),
            engineer(2, "core", &[Skill::Rust { years: 1 }], "*"),
            engineer(
                3,
                "web",
                &[Skill::Go { years: 3 }, Skill::Design, Skill::Design],
                "ghent",
            ),
            engineer(4, "web", &[], "paris"),
        ])
        .unwrap()
    }

    fn ids(result: Option<EngineerDBQueryResult<'_>>) -> Vec<u32> {
        result
            .map(|result| result.iter().map(|engineer| engineer.id).collect())
            .unwrap_or_default()
    }

    #[test]
    fn test_set_filter() {
        let db = db();
        let mut query = db.query();
        query.skills(Skill::Design);
        assert_eq!(ids(query.execute()), [1, 3]);
        query.skills(Skill::Rust { years: 1 });
        assert_eq!(ids(query.execute()), [1, 2, 3]);
        query.reset().skills(Skill::Go { years: 9 });
        assert_eq!(ids(query.execute()), []);
        query.reset().tags("early").tags("web");
        assert_eq!(ids(query.execute()), [1, 2]);
        query.remote(true);
        assert_eq!(ids(query.execute()), [2]);
    }

    #[test]
    fn test_optional_set_filter() {
        let db = db();
        let mut query = db.query();
        query.floors(9);
        assert_eq!(ids(query.execute()), [1, 3, 4]);
        query.reset().floors(3);
        assert_eq!(ids(query.execute()), [3]);
        query.reset().floors(2);
        assert_eq!(ids(query.execute()), []);
    }

    #[test]
    fn test_set_filter_members() {
        let db = db();
        let mut query = db.query();
        query.projects(7u16);
        assert_eq!(ids(query.execute()), [1, 2, 3, 4]);
        query.reset().projects(30u16).projects(40u16);
        assert_eq!(ids(query.execute()), [3, 4]);
        query.reset().languages('f');
        assert_eq!(ids(query.execute()), [2, 3]);
        query.languages('d').skills(Skill::Design);
        assert_eq!(ids(query.execute()), [3]);
    }

    #[test]
    fn test_filter_where() {
        let db = db();
        let mut query = db.query();
        query.skills_where(|skill| matches!(skill, Skill::Rust { .. }));
        assert_eq!(ids(query.execute()), [1, 2]);
        query.skills_where(|skill| matches!(skill, Skill::Rust { years } if *years > 2));
        assert_eq!(ids(query.execute()), [1]);
        query.reset().skills_where(|_| false);
        assert_eq!(ids(query.execute()), []);
        let mut calls = 0;
        query.reset().skills_where(|_| {
            calls += 1;
            true
        });
        assert_eq!(ids(query.execute()), [1, 2, 3]);
        // once per distinct value, not per row
        assert_eq!(calls, 4);
        // wildcard rows match whatever is asked for
        query.reset().office_where(|office| office.starts_with('p'));
        assert_eq!(ids(query.execute()), [2, 4]);
    }

    #[test]
    fn test_any_by_function() {
        let db = db();
        let mut query = db.query();
        query.office("ghent");
        assert_eq!(ids(query.execute()), [1, 2, 3]);
        query.reset().office("tokyo");
        assert_eq!(ids(query.execute()), [2]);
        query.reset().office("*");
        assert_eq!(ids(query.execute()), [2]);
        query.reset().desk("a1");
        assert_eq!(ids(query.execute()), [1, 2, 3]);
        query.reset().desk("b2");
        assert_eq!(ids(query.execute()), [1]);
    }

    #[test]
    fn test_key_by_function() {
        let db = db();
        assert_eq!(db.get_by_badge("core-2").unwrap().id, 2);
        assert_eq!(db.index_by_badge("web-4"), Some(3));
        assert!(db.get_by_badge("").is_none());
        assert_eq!(db.index_by_id(&3), Some(2));
        assert_eq!(db.index_by_id(&9), None);
        assert_eq!(db.get(2).unwrap().id, 3);
        assert!(db.get(4).is_none());

        let mut db = db;
        let err = db.append(engineer(12, "core", &[], "x")).map(|_| ());
        assert!(err.is_ok());
        let mut duplicate = engineer(2, "core", &[], "x");
        duplicate.id = 10;
        let err = db.append(duplicate).unwrap_err();
        assert_eq!(err.kind(), EngineerDBErrorKind::DuplicateKey);
        assert_eq!(db.len(), 5);
        assert!(db.get_by_id(&10).is_none());
    }

    #[test]
    fn test_columns() {
        let mut db = db();
        let core = db.column(|engineer| engineer.team == "core");
        let busy = db.runtime_column();
        assert_eq!(busy.len(), 4);

        let mut query = db.query();
        query.and(Filter::Is(&core));
        assert_eq!(ids(query.execute()), [1, 2]);
        query.reset().and(Filter::Not(&core)).office("ghent");
        assert_eq!(ids(query.execute()), [3]);

        query.reset().and(Filter::Runtime(&busy, false));
        assert_eq!(ids(query.execute()), [1, 2, 3, 4]);
        assert!(busy.claim(db.index_by_id(&2).unwrap()));
        assert!(!busy.claim(db.index_by_id(&2).unwrap()));
        assert_eq!(ids(query.execute()), [1, 3, 4]);
        query.reset().and(Filter::Runtime(&busy, true));
        assert_eq!(ids(query.execute()), [2]);
        query.and(Filter::Is(&core)).remote(true);
        assert_eq!(ids(query.execute()), [2]);
        query.remote(false);
        assert_eq!(ids(query.execute()), []);

        // rows added later are in neither column
        db.append(engineer(5, "core", &[], "ghent")).unwrap();
        let mut query = db.query();
        query.and(Filter::Is(&core));
        assert_eq!(ids(query.execute()), [1, 2]);
        query.reset().and(Filter::Runtime(&busy, false));
        assert_eq!(ids(query.execute()), [1, 3, 4, 5]);
    }

    #[test]
    fn test_result() {
        let mut db = db();
        db.shrink_to_fit();
        let mut query = db.query();
        query.office("ghent");
        let result = query.execute().unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result.first().id, 1);
        for _ in 0..20 {
            assert_eq!(result.any_where(|engineer| engineer.remote).unwrap().id, 2);
            assert!([1, 2, 3].contains(&result.any().id));
        }
        assert!(result.any_where(|engineer| engineer.id > 3).is_none());
        let filtered = result.filter(|engineer| !engineer.remote).unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(ids(Some(filtered)), [1, 3]);
        assert!(result.filter(|_| false).is_none());
    }

    #[test]
    fn test_failed_append_leaves_no_trace() {
        let mut db = db();
        let duplicate = engineer(2, "new", &[Skill::Design], "tokyo");
        assert!(db.append(duplicate).is_err());
        let rest = vec![
            engineer(5, "new", &[Skill::Design], "tokyo"),
            engineer(1, "new", &[Skill::Design], "tokyo"),
            engineer(6, "new", &[Skill::Design], "tokyo"),
        ];
        assert!(db.extend(rest).is_err());
        assert_eq!(db.len(), 5);

        let mut query = db.query();
        query.skills(Skill::Design);
        assert_eq!(ids(query.execute()), [1, 3, 5]);
        query.reset().office("tokyo");
        assert_eq!(ids(query.execute()), [2, 5]);
        query.reset().remote(false);
        assert_eq!(ids(query.execute()), [1, 3, 5]);
        query.reset().projects(50u16);
        assert_eq!(ids(query.execute()), [5]);
        query.reset().projects(7u16);
        assert_eq!(ids(query.execute()), [1, 2, 3, 4, 5]);
        assert_eq!(db.index_by_badge("new-5"), Some(4));
        assert!(db.get_by_badge("new-6").is_none());
    }

    #[test]
    fn test_with_capacity() {
        let mut db = EngineerDB::with_capacity(8);
        assert!(db.capacity() >= 8);
        assert!(db.is_empty());
        db.append(engineer(1, "core", &[], "ghent")).unwrap();
        assert_eq!(db.len(), 1);
        assert_eq!(db.get_by_id(&1).unwrap().team, "core");
    }

    #[test]
    fn test_value_and_where_are_both_required() {
        let db = db();
        let mut query = db.query();
        query
            .skills(Skill::Design)
            .skills_where(|skill| matches!(skill, Skill::Go { .. }));
        assert_eq!(ids(query.execute()), [3]);
        // a row without value matches no predicate, a wildcard row matches all
        query.reset().desk_where(|_| true);
        assert_eq!(ids(query.execute()), [1, 2, 3]);
        query.reset().desk_where(|_| false);
        assert_eq!(ids(query.execute()), [1]);
        query.reset().floors(2).floors(9);
        assert_eq!(ids(query.execute()), [1, 3, 4]);
    }

    #[test]
    fn test_any_where_is_random() {
        let db = db();
        let result = db.query().execute().unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..200 {
            seen.insert(result.any_where(|engineer| engineer.id != 2).unwrap().id);
        }
        assert!(seen.into_iter().eq([1, 3, 4]));
    }

    #[test]
    fn test_derive_any() {
        use venndb::Any;

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Any)]
        enum Place<T> {
            #[any]
            Anywhere,
            #[any]
            Near {
                _km: u8,
            },
            City(T),
            #[any]
            Region(T, T),
        }

        assert!(Place::<u8>::Anywhere.is_any());
        assert!(Place::<u8>::Near { _km: 3 }.is_any());
        assert!(Place::Region(1, 2).is_any());
        assert!(!Place::City(1).is_any());

        #[derive(Debug, VennDB)]
        struct Shop {
            #[venndb(filter, any)]
            place: Place<&'static str>,
        }

        let db = ShopDB::from_rows(vec![
            Shop {
                place: Place::City("ghent"),
            },
            Shop {
                place: Place::Anywhere,
            },
            Shop {
                place: Place::City("paris"),
            },
        ]);
        let mut query = db.query();
        query.place(Place::City("ghent"));
        assert_eq!(query.execute().unwrap().len(), 2);
        query.reset().place(Place::Near { _km: 1 });
        assert_eq!(query.execute().unwrap().len(), 1);
    }

    #[test]
    fn test_unknown_value_next_to_known() {
        let db = db();
        let mut query = db.query();
        query.skills(Skill::Go { years: 9 }).skills(Skill::Design);
        assert_eq!(ids(query.execute()), [1, 3]);
    }

    #[test]
    fn test_many_rows() {
        #[derive(Debug, VennDB)]
        struct Point {
            #[venndb(key)]
            id: u32,
            even: bool,
            third: Option<bool>,
            #[venndb(filter)]
            bucket: u32,
            #[venndb(filter, set)]
            divisors: BTreeSet<u32>,
        }

        let rows = 20_000u32;
        let db = PointDB::from_iter((0..rows).map(|id| {
            Point {
                id,
                even: id % 2 == 0,
                third: (id % 5 != 0).then_some(id % 3 == 0),
                bucket: id % 1000,
                divisors: [2, 3, 5, 7, 11]
                    .into_iter()
                    .filter(|d| id % d == 0)
                    .collect(),
            }
        }))
        .unwrap();

        let check = |query: &PointDBQuery<'_>, expected: &dyn Fn(u32) -> bool| {
            let found: Vec<u32> = query
                .execute()
                .map(|result| result.iter().map(|point| point.id).collect())
                .unwrap_or_default();
            let expected: Vec<u32> = (0..rows).filter(|id| expected(*id)).collect();
            assert_eq!(found, expected);
        };

        let mut query = db.query();
        check(&query, &|_| true);
        check(query.even(true), &|id| id % 2 == 0);
        check(query.third(false), &|id| {
            id % 2 == 0 && id % 5 != 0 && id % 3 != 0
        });
        check(query.bucket(6u32), &|id| {
            id % 1000 == 6 && id % 5 != 0 && id % 3 != 0
        });
        check(query.reset().bucket(6u32).bucket(7u32).even(false), &|id| {
            id % 1000 == 7
        });
        check(query.reset().divisors(7u32).divisors(11u32), &|id| {
            id % 7 == 0 || id % 11 == 0
        });
        check(query.third(true).even(false), &|id| {
            (id % 7 == 0 || id % 11 == 0) && id % 5 != 0 && id % 3 == 0 && id % 2 == 1
        });
        check(
            query
                .reset()
                .divisors_where(|d| *d > 5)
                .bucket_where(|b| *b < 3),
            &|id| (id % 7 == 0 || id % 11 == 0) && id % 1000 < 3,
        );
    }
}

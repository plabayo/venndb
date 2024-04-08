#![allow(dead_code)]

use venndb::VennDB;

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    #[venndb(key)]
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: bool,
    #[venndb(filter)]
    department: Department,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    fn test_duplicate_key_with_zero_polution() {
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

        // ensure there was no polution,
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

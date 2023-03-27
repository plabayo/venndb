![flagdb banner](./docs/img/banner.png)

[![Build Status][build-status]][build-url]
[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/flagdb.svg
[crates-url]: https://crates.io/crates/flagdb
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/plabayo/flagdb/blob/main/LICENSE
[build-status]: https://github.com/plabayo/flagdb/actions/workflows/CI.yml/badge.svg?branch=main
[build-url]: https://github.com/plabayo/flagdb/actions/workflows/CI.yml

[Github Repository](https://github.com/plabayo/flagdb) |
[API Docs](https://docs.rs/flagdb)

An in-memory database in Rust for rows queried using bit (flag) columns.
This database is designed for a very specific use case where you have mostly static data that you typically load at startup and have to query constantly using very simple filters. Datasets
like these can be large and should be both fast and compact.

It is developed mostly to scratch our own itch and there are certainly viable alternatives
available.

Contributions and feedback are welcome.

## Example

### Read Only Database Example

```rust
use flagdb::flagdb;

flagdb! {
    struct Employee {
        id: u32,
        name: String,
        is_manager: bool,
        is_admin: bool,
        is_active: bool,
        department: enum {
            Sales,
            Marketing,
            Engineering,
            HumanResources,
            Accounting,
        }
    }
}

fn main() {
    let db = EmployeeDB::with_capacity(100).extend(vec![
        Employee {
            id: 1,
            name: "John".to_string(),
            is_manager: true,
            is_admin: false,
            is_active: true,
            department: Department::Sales,
        },
        Employee {
            id: 2,
            name: "Jane".to_string(),
            is_manager: false,
            is_admin: true,
            is_active: true,
            department: Department::Marketing,
        },
        Employee {
            id: 3,
            name: "Bob".to_string(),
            is_manager: false,
            is_admin: false,
            is_active: false,
            department: Department::Engineering,
        },
    ]);

    let active_managers: Vec<String> = db.query()
        .is_manager()
        .is_active()
        .run()
        .map(|employee| employee.name)
        .collect();

    assert_eq!(active_managers, vec!["John".to_string()]);
}
```

The above example show how easy it is to create an in-memory database
with little effort while still being powerful and easy to use.

What's going om under the hood? High level the following is generated
(use `cargo expand` to see the fully generated code):

```rust
#[derive(Debug, flagdb::Serialize, flagdb::Deserialize)]
struct Employee {
    pub id: u32,
    pub name: String,
    pub is_manager: bool,
    pub is_admin: bool,
    pub is_active: bool,
    pub department: EmployeeDepartment,
}

#[derive(Debug, flagdb::Serialize, flagdb::Deserialize)]
enum EmployeeDepartment {
    Sales,
    Marketing,
    Engineering,
    HumanResources,
    Accounting,
}

#[derive(Debug)]
struct EmployeeDB {
    employees: Vec<Employee>,
    is_manager: flagdb::BitVec,
    is_admin: flagdb::BitVec,
    is_active: flagdb::BitVec,
    departments: flagdb::BitMap<EmployeeDepartment>,
}

struct EmployeeDBQueryBuilder<'a> {
    db: &'a EmployeeDB,

    ...
}

impl EmployeeDB {
    fn new() -> Self { ... }
    fn with_capacity(capacity: usize) -> Self { ... }

    fn extend(&mut self, employees: impl IntoIterator<Item = Employee>) { ... }

    fn query(&self) -> EmployeeDBQueryBuilder { ... }

    fn iter(&self) -> impl Iterator<Item = &Employee> {
        self.employees.iter()
    }
}

impl EmployeeDBQueryBuilder<'_> {
    fn is_manager(&mut self) -> &mut Self { ... }
    fn is_admin(&mut self) -> &mut Self { ... }
    fn is_active(&mut self) -> &mut Self { ... }
    fn department(&mut self, department: EmployeeDepartment) -> &mut Self { ... }

    fn run(self) -> impl Iterator<Item = &Employee> { ... }
}

impl Iterator for EmployeeDB<'_> {
    type Item = &'_ Employee;

    fn next(&mut self) -> Option<Self::Item> { ... }
}

impl IntoIterator for EmployeeDB {
    type Item = Employee;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.employees.into_iter()
    }
}
```

### Mutate Example

Should you need to mutate the database, you can do so with
a couple of changes.

First the flagdb creation has to be change a bit:

flagdb! {
    struct Employee {
        #[flagdb::key]
        id: u32,
        name: String,
        is_manager: bool,
        is_admin: bool,
        is_active: bool,
        department: enum {
            Sales,
            Marketing,
            Engineering,
            HumanResources,
            Accounting,
        }
    }
}

Note the use of the `#[flagdb::key]` attribute on the `id` field,
which will ensure that internally we generate a `flagdb::FxHashMap<u32, usize>`
property in the actual `EmployeeDB` struct.

Then we can mutate previously queried employees as follows:

```rust
db.get_mut(1).unwrap().is_manager = false; // John is no longer a manager
```

Without modifications we can however also query mutable:

```rust
// fire all the managers
let active_managers: Vec<&mut Employee> = db.mutate()
    .is_manager()
    .is_active()
    .run()
    .map(|employee| employee.is_active = false);
```

## Safety

These crates uses `#![forbid(unsafe_code)]` to ensure everything is implemented in
100% safe Rust.

> The exception is [flagdb-chrome](flagdb-chrome) as we bind there to C++ code using CXX.

## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][contributing] to help you get involved in the
`flagdb` project.

## License

This project is licensed under the [MIT license][LICENSE].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `flagdb` by you, shall be licensed as MIT, without any
additional terms or conditions.

[contributing]: https://github.com/plabayo/flagdb/blob/main/CONTRIBUTING.md
[license]: https://github.com/plabayo/flagdb/blob/main/flagdb/LICENSE

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.1.1 (2024-04-10)

Non-Breaking Changes:

* fix clippy linter warning errors (`missing_docs`): `missing documentation for a variant`;

# 0.1.0 (2024-04-09)

Implement the first version of this library, `venndb`.
Released as `venndb` (0.1.0) and `venndb-macros` (0.1.0).

API Example:

```rust
use venndb::VennDB

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    #[venndb(skip)]
    foo: bool,
    #[venndb(filter)]
    department: Department,
}

fn main() {
    let db = EmployeeDB::from_iter(/* .. */);

    let mut query = db.query();
    let employee = query
        .is_admin(true)
        .is_manager(false)
        .department(Department::Engineering)
        .execute()
        .expect("to have found at least one")
        .any();

    println!("non-manager admin engineer: {:?}", employee);
}
```

This API, using nothing more then a `derive` macro allows to:

- store rows of data in a generated `database`;
- query the database using the defined `filter` fields;
- get references to rows directly by a `key`;

The crate comes with examples and a detailed README.

Please share with us if you have any feedback about this first version,
how you are using it, what you would like different, etc.

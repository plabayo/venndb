# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.2.1 (2024-04-15)

Non-Breaking changes:

* support `#[venndb(any)]` filters;
  * these are possible only for `T` filter maps, where `T: ::venndb::Any`;
  * `bool` filters cannot be `any` as `bool` doesn't implement the `::venndb::Any` trait;
  * rows that are `any` will match regardless of the query filter used for that property;

# 0.2.0 (2024-04-15)

Breaking Changes:

* Support Option<T> in a special way:
  * for filters it means that both positive and negative bits will be set to false if the value is `None`;
  * for filter maps this means that the filter is not even registered;
  * keys cannot be optional;
    * While technically this is a breaking change it is not expected to actually break someone,
      as keys always had to be unique already and two times `None` will result in same hash... so it is unlikely
      that there was an `Option<T>` already used by someone;
  * this is potentially breaking as some implementations from `0.1*` might have already used `Option` in a different way;

While this changes behaviour of `filters` and `filter maps` it is unlikely that someone was already using
`Option<T>` for these types before, as their ergonomics have been a bit weird prior to this version.
Even more so for `filter maps` it could have resulted in panics.

Non-Breaking Changes:

* improve documentation;

Updated Example from 0.1:

```rust
use venndb::VennDB

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: Option<bool>,
    is_admin: bool,
    #[venndb(skip)]
    foo: bool,
    #[venndb(filter)]
    department: Department,
    #[venndb(filter)]
    country: Option<String>,
}

fn main() {
    let db = EmployeeDB::from_iter(/* .. */);

    let mut query = db.query();
    let employee = query
        .is_admin(true)
        .is_manager(false)  // rows which have `None` for this property will NOT match this filter
        .department(Department::Engineering)
        .execute()
        .expect("to have found at least one")
        .any();

    println!("non-manager admin engineer: {:?}", employee);
    // as we didn't specify a `country` filter, even rows without a country specified will
    // match here if they match the defined (query) filters)
}
```

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

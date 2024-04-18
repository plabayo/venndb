# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.4.0 (2024-04-19)

Breaking Changes:

* [[`#7`](https://github.com/plabayo/venndb/issues/7)]: correct the behaviour of any filter map query values:
  - When using an any value as a query filter map value it will now only match rows
    which have an any value registered for the row;
  - Prior to this release it was matching on all rows, as if the filter wasn't defined.
    This seemed correct when deciding on it, but on hindsight is is incorrect behaviour.

New Features:

* [[`#8`](https://github.com/plabayo/venndb/issues/8)]: support custom validations of rows prior to appending them

Example:

```rust
#[derive(Debug, VennDB)]
#[venndb(name = "MyDB", validatator = "my_validator_fn")]
pub struct Value {
   pub foo: String,
   pub bar: u32,
}

fn my_validator_fn(value: &Value) -> bool {
    !foo.is_empty() && value.bar > 0
}

let mut db = MyDB::default();
assert!(db.append(Value {
    foo: "".to_owned(),
    bar: 42,
}).is_err()); // fails because foo == empty
```

# 0.3.0 (2024-04-18)

Breaking Changes:

* [[`#6`](https://github.com/plabayo/venndb/issues/6)] query filter maps now accept arguments as `impl Into<T>` instead of `T`,
  this can be a breaking change for users that were inserting them as `value.into()`,
  as the compiler will for these cases now give a compile time error due to the now introduced ambiguity;
  * Migration is as easy as removing the manual `.into()` (like) calls that you previously had to add yourself;

Bug Fixes from [0.2.1](#021-2024-04-15):

* [[`#5`](https://github.com/plabayo/venndb/issues/5)] any filters now also allow rows to match on unknown filter map variants.
  * e.g. if you have a `MyType` filter map and have not a single row that has `"foo"` as value,
    then all rows that that have a value for which `assert!(Any::is_any(value: MyType))` will still work.
  * prior to this bug fix these values could not be matched on, and the `any` rows would only hit
    if there were also rows that had that value explicitly defined.

# 0.2.1 (2024-04-15)

A backwards compatible patch for [v0.2.0](#020-2024-04-15),
to support rows that allow any value for a specific column.

Non-Breaking changes:

* support `#[venndb(any)]` filters;
  * these are possible only for `T` filter maps, where `T: ::venndb::Any`;
  * `bool` filters cannot be `any` as `bool` doesn't implement the `::venndb::Any` trait;
  * rows that are `any` will match regardless of the query filter used for that property;

Example usage:

```rust
use venndb::{Any, VennDB};
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Department {
  Any,
  Hr,
  Engineering,
}

impl Any for Department {
  fn is_any(&self) -> bool {
    self == Department::Any
  }
}

#[derive(Debug, VennDB)]
pub struct Employee {
  name: String,
  #[venndb(filter, any)]
  department: Department,
}

let db = EmployeeDB::from_iter([
  Employee { name: "Jack".to_owned(), department: Department::Any },
  Employee { name: "Derby".to_owned(), department: Department::Hr },
]);
let mut query = db.query();

// will match Jack and Derby, as Jack is marked as Any, meaning it can work for w/e value
let hr_employees: Vec<_> = query.department(Department::Hr).execute().unwrap().iter().collect();
assert_eq!(hr_employees.len(), 2);
```

In case you combine it with the filter map property being optional (`department: Option<Department>`),
then it will still work the same, where rows with `None` are seen as nothing at all and just ignored.
This has no affect on the correct functioning of `Any`.

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

Options, be it filters of filter maps, allow you to have rows that do not register any value for optional
properties, allowing them to exist without affecting the rows which do have it.

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

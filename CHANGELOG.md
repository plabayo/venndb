# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.7.0 (2026-11-13)

A new engine, with the same API on top of it.

Breaking Changes:

* Bump MSRV to 1.96;
* `venndb::__internal` is gone, the types a database is made of are now public:
  `RowSet`, `KeyIndex`, `BoolColumn`, `ValueColumn`, `Plan`, `Filter`, `Matches` and `RuntimeColumn`;
* `<DB>QueryResult::iter` now returns `impl Iterator`;
  * the `<DB>QueryResultIter` types are gone, which only breaks you if you named them;
* a database holds at most `u32::MAX` rows;
* `<DB>Query` and `<DB>QueryResult` are now generated for every database, also for one without filters;
* filters can no longer be named `and`, `any` or `any_where`, or `<filter>_where` and `<filter>_ref`
  for a `filter` that exists as well,
  as query methods of that name are now generated;
  * Migration is as easy as renaming the property, or to `#[venndb(skip)]` it;
* querying a filter map on a value no row has, next to values that do have rows,
  now returns the rows of the latter;
  * Prior to this release it returned nothing at all.

New Features:

* [[`#17`](https://github.com/plabayo/venndb/issues/17)]: support filter maps with many values:
  * values are stored as a list of rows as long as few rows have them,
    and as a bitmask only once many do;
  * a database of 100 000 rows with 16 000 distinct values is created in 12 milliseconds
    instead of 5 seconds, and takes 6.5 MB instead of 330 MB;
* support properties with multiple values, of which a row matches each: `#[venndb(filter, set)]`;
  * for `HashSet`, `BTreeSet` and types implementing `venndb::FilterSet`;
  * or any other type using `#[venndb(filter, members = path)]`;
* support filtering on what the values of a property look like: `<DB>Query::<property>_where(predicate)`;
  * the predicate is called once per distinct value, not once per row;
  * rows with an _any_ value always match;
* [[`#16`](https://github.com/plabayo/venndb/issues/16)]: support columns that are no property of a row:
  * `<DB>::column(predicate)` is computed once;
  * a `venndb::RuntimeColumn` can be changed at any time and from any thread,
    and grows along with the database;
  * add them to a query with `<DB>Query::and`;
* [[`#21`](https://github.com/plabayo/venndb/issues/21)]: support keys computed from the row by a function: `#[venndb(key = path)]`;
* support wildcards decided by a function instead of `venndb::Any`: `#[venndb(filter, any = path)]`;
* add `#[derive(Any)]` to implement `venndb::Any` for an enum, with `#[any]` on the variants that match any value;
* add `<DB>Query::any()` and `<DB>Query::any_where(predicate)` to pick a random row without
  finding all rows first: random rows are tried if many are expected to match;
  * three `bool` filters on 1 000 000 rows: 0.2 microseconds instead of 12;
* add `<DB>Query::<property>_ref(&value)` to filter on a value without giving it away;
* add `<DB>QueryResult::any_where(predicate)` and `<DB>QueryResult::len()`;
* add `<DB>::index_by_<key>`, `<DB>::get` and `<DB>::shrink_to_fit`;
* add the `derive` feature, enabled by default, to be able to use the engine without the macro.

Example:

```rust
use venndb::{Filter, VennDB};

#[derive(Debug, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    #[venndb(filter, set)]
    skills: HashSet<Skill>,
    #[venndb(filter, any = Country::is_any)]
    country: Country,
}

let db = EmployeeDB::from_rows(/* .. */).unwrap();
let on_leave = RuntimeColumn::new();
on_leave.set(db.index_by_id(&42).unwrap(), true);

let mut query = db.query();
query
    .skills_where(|skill| matches!(skill, Skill::Language { native: true, .. }))
    .country("BE")
    .and(Filter::Runtime(&on_leave, false));
let employee = query.execute().unwrap().any();
```

Other Changes:

* Replace `hashbrown` and `rand` crate deps with the standard library;
  * `foldhash` is used as the hash function, the one dependency left next to the macro;
* Queries count and pick rows a machine word at a time, and start from the smallest set of rows;
* Rewrite benchmarks around a database shaped like the one of `rama`,
  compared with `roaring` and an indexed `sqlite`;
* The fuzzer now compares queries with a scan of the rows;
* Add examples, and test that the code shown in the README and on the website is part of them.

# 0.6.1 (2025-11-18)

Other Changes:

* Update dependencies (incl major update from Hashbrown from 0.15 to 0.16)
* Update homepage from venndb.rs to venndb.plabayo.tech (save on registrar costs)

# 0.6.0 (2025-08-23)

Breaking Changes:

* Bump MSRV to 1.88 and rust edition 2024;

Other Changes:

* Replace `bitvec` crate dep with internal custom impl;
  * `bitvec` was no longer maintained and starting
    to cause issues on nightly;

# 0.5.0 (2024-05-23)

New Features:

- [[`#15`](https://github.com/plabayo/venndb/issues/15)]: support multi dimensional filter options to be filtered in group

Example:

```rust
use venndb::{Any, VennDB};

#[derive(Debug, VennDB)]
pub struct Value {
   #[venndb(filter)]
   pub foo: String,
   pub bar: u32,
}

let db = ValueDB::from_iter([
    Value {
        foo: "a".to_owned(),
        bar: 8,
    },
    Value {
        foo: "b".to_owned(),
        bar: 12,
    },
    Value {
        foo: "c".to_owned(),
        bar: 16,
    },
].into_Iter()).unwrap();

let mut query = db.query();
query.foo(MyString("a".to_owned()));
query.foo(MyString("c".to_owned()));
let values: Vec<_> = query.execute().unwrap().iter().collect();
assert_eq!(values.len(), 2);
assert_eq!(values[0].bar, 8);
assert_eq!(values[0].bar, 16);
```

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
#[venndb(name = "MyDB", validator = my_validator_fn)]
pub struct Value {
   pub foo: String,
   pub bar: u32,
}

fn my_validator_fn(value: &Value) -> bool {
    !value.foo.is_empty() && value.bar > 0
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

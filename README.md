# VennDB

An **append-only** in-memory database in Rust for rows queried using bit (flag) columns.
This database is designed for a very specific use case where you have mostly static data that you typically load at startup and have to query constantly using very simple filters. Datasets
like these can be large and should be both fast and compact.

For the limited usecases where `venndb` can be applied to,
it has less dependencies and is faster then traditional choices,
such as a naive implementation or a more heavy lifted dependency such as _Sqlite_.

> See [the benchmarks](#benchmarks) for more information on this topic.

This project was developed originally in function of [`rama`](https://ramaproxy.org),
where you can see it being used for example to provide an in-memory (upstream) proxy database.
Do let us know in case you use it as well in your project, such that we can assemble a showcase list.

![venndb banner](https://raw.githubusercontent.com/plabayo/venndb/main/docs/img/banner.svg)

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT License][license-mit-badge]][license-mit-url]
[![Apache 2.0 License][license-apache-badge]][license-apache-url]
[![rust version][rust-version-badge]][rust-version-url]
[![Build Status][actions-badge]][actions-url]

[![Discord][discord-badge]][discord-url]
[![Buy Me A Coffee][bmac-badge]][bmac-url]
[![GitHub Sponsors][ghs-badge]][ghs-url]

[crates-badge]: https://img.shields.io/crates/v/venndb.svg
[crates-url]: https://crates.io/crates/venndb
[docs-badge]: https://img.shields.io/docsrs/venndb/latest
[docs-url]: https://docs.rs/venndb/latest/venndb/index.html
[license-mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-mit-url]: https://github.com/plabayo/venndb/blob/main/LICENSE-MIT
[license-apache-badge]: https://img.shields.io/badge/license-APACHE-blue.svg
[license-apache-url]: https://github.com/plabayo/venndb/blob/main/LICENSE-APACHE
[rust-version-badge]: https://img.shields.io/badge/rustc-1.75+-blue?style=flat-square&logo=rust
[rust-version-url]: https://www.rust-lang.org
[actions-badge]: https://github.com/plabayo/venndb/workflows/CI/badge.svg
[actions-url]: https://github.com/plabayo/venndb/actions

[discord-badge]: https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white
[discord-url]: https://discord.gg/29EetaSYCD
[bmac-badge]: https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black
[bmac-url]: https://www.buymeacoffee.com/plabayo
[ghs-badge]: https://img.shields.io/badge/sponsor-30363D?style=for-the-badge&logo=GitHub-Sponsors&logoColor=#EA4AAA
[ghs-url]: https://github.com/sponsors/plabayo

💬 Come join us at [Discord][discord-url] on the `#venndb` public channel. To ask questions, discuss ideas and ask how venndb may be useful for you.

## Index

`venndb` manual:

- [Usage](#usage): quick introduction on how to use `venndb`;
- [Benchmarks](#benchmarks): benchmark results to give you a rough idea how `venndb` peforms for the use case it is made for (write once, read constantly, using binary filters mostly);
- [Q&A](#qa): Frequently Asked Questions (FAQ);
- [Example](#example): the full example (expanded version from [Usage](#usage)), tested and documented;
- [Generated Code Summary](#generated-code-summary): a documented overview of the API that `venndb` will generate for you when using `#[derive(VennDB)]` on your _named field struct_;

technical information:

- [Safety](#--safety)
- [Compatibility](#--compatibility)
- [MSRV](#minimum-supported-rust-version) (older versions might work but we make no guarantees);
- [Roadmap](#--roadmap)
- [License](#--license): [MIT license][mit-license] and [Apache 2.0 License][apache-license]

misc:

- [Contributing](#--contributing)
- [Sponsors](#--sponsors)

## Usage

Add `venndb` as a dependency:

```sh
cargo add venndb
```

and import the `derive` macro in the module where you want to use it:

```rust,ignore
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
    #[venndb(filter, any)]
    department: Department,
    #[venndb(filter)]
    country: Option<String>,
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

See [the full example](#example) or [the "Generated Code Summary" chapter](#generated-code-summary) below
to learn how to use the `VennDB` and its generated code.

## Benchmarks

Benchmarks displayed here are taken on a dev machine with following specs:

```text
Macbook Pro — 16 inch (2023)
Chip: Apple M2 Pro
Memory: 16 GB
OS: Sonoma 14.2
```

The benchmarks tests 3 different implementations of a proxy database

- `venndb` version (very similar to [the example below](#example))
- a `naive` version, which is just a `Vec<Proxy>`, over which is iterated
- an `sqlite` version (using [the `sqlite` crate (version: `0.34.0`)](https://docs.rs/sqlite/0.34.0/sqlite/))

The benchmarks are created by:

1. running `just bench`;
2. copying the output into [./scripts/plot_bench_charts](./scripts/plot_bench_charts.py) and running it.

Snippet that is ran for each 3 implementations:

```rust,ignore
fn test_db(db: &impl ProxyDB) {
    let i = next_round();

    let pool = POOLS[i % POOLS.len()];
    let country = COUNTRIES[i % COUNTRIES.len()];

    let result = db.get(i as u64);
    divan::black_box(result);

    let result = db.any_tcp(pool, country);
    divan::black_box(result);

    let result = db.any_socks5_isp(pool, country);
    divan::black_box(result);
}
```

### Benchmark Performance Results

Performance for Database with `100` records:

| Proxy DB | Fastest (µs) | Median (µs) | Slowest (µs) |
| --- | --- | --- | --- |
| naive_proxy_db_100             | 6.50 | 8.00 | 18.04 |
| sql_lite_proxy_db_100          | 32.58 | 37.37 | 302.00 |
| venn_proxy_db_100              | 0.89 | 0.92 | 2.74 |

Performance for Database with `12_500` records:

| Proxy DB | Fastest (µs) | Median (µs) | Slowest (µs) |
| --- | --- | --- | --- |
| naive_proxy_db_12_500          | 404.00 | 407.70 | 478.70 |
| sql_lite_proxy_db_12_500       | 1061.00 | 1073.00 | 1727.00 |
| venn_proxy_db_12_500           | 16.04 | 16.97 | 25.54 |

Performance for Database with `100_000` records:

| Proxy DB | Fastest (µs) | Median (µs) | Slowest (µs) |
| --- | --- | --- | --- |
| naive_proxy_db_100_000         | 3790.00 | 3837.00 | 5731.00 |
| sql_lite_proxy_db_100_000      | 8219.00 | 8298.00 | 9424.00 |
| venn_proxy_db_100_000          | 124.20 | 129.20 | 156.30 |

We are not database nor hardware experts though. Please do open an issue if you think
these benchmarks are incorrect or if related improvements can be made.
Contributions in the form of Pull requests are welcomed as well.

See [the Contribution guidelines](#contribution) for more information.

### Benchmark Allocations Results

Allocations for Database with `100` records:

| Proxy DB | Fastest (KB) | Median (KB) | Slowest (KB) |
| --- | --- | --- | --- |
| naive_proxy_db_100             | 0.33 | 0.33 | 0.33 |
| sql_lite_proxy_db_100          | 4.04 | 4.04 | 4.04 |
| venn_proxy_db_100              | 0.05 | 0.05 | 0.05 |

Allocations for Database with `12_500` records:

| Proxy DB | Fastest (KB) | Median (KB) | Slowest (KB) |
| --- | --- | --- | --- |
| naive_proxy_db_12_500          | 40.73 | 40.73 | 40.73 |
| sql_lite_proxy_db_12_500       | 5.03 | 5.02 | 5.03 |
| venn_proxy_db_12_500           | 3.15 | 3.15 | 3.15 |

Allocations for Database with `100_000` records:

| Proxy DB | Fastest (KB) | Median (KB) | Slowest (KB) |
| --- | --- | --- | --- |
| naive_proxy_db_100_000         | 323.30 | 323.30 | 323.70 |
| sql_lite_proxy_db_100_000      | 5.02 | 5.02 | 5.01 |
| venn_proxy_db_100_000          | 25.02 | 25.02 | 25.02 |

We are not database nor hardware experts though. Please do open an issue if you think
these benchmarks are incorrect or if related improvements can be made.
Contributions in the form of Pull requests are welcomed as well.

See [the Contribution guidelines](#contribution) for more information.

## Q&A

> ❓ Why use this over Database X?

`venndb` is not a database, but is close enough for some specific purposes. It shines for long-lived read-only use cases where you need to filter on plenty of binary properties and get a rando matching result.

Do not try to replace your usual database needs with it.

> ❓ Where can I propose a new feature X or some other improvement?

Please [open an issue](https://github.com/plabayo/venndb/issues) and also read [the Contribution guidelines](#contribution). We look forward to hear from you.

Alternatively you can also [join our Discord][discord-url] and start a conversation / discussion over there.

> ❓ Can I use _whatever_ type for a `#[venndb(filter)]` property?

Yes, as long as it implements `PartialEq + Eq + Hash + Clone`.
That said, we do recommend that you use `enum` values if you can, or some other highly restricted form.

Using for example a `String` directly is a bad idea as that would mean that `bE` != `Be` != `BE` != `Belgium` != `Belgique` != `België`. Even though these are really referring all to the same country. In such cases a much better idea is to at the very least create a wrapper type such as `struct Country(String)`, to allow you to enforce sanitization/validation when creating the value and ensuring the hashes will be the same for those values that are conceptually the same.

> ❓ How do I make a filter optional?

Both filters (`bool` properties) and filter maps (`T != bool` properties with the `#[venndb(filter)]` attribute)
can be made optional by wrapping the types with `Option`, resulting in `Option<bool>` and `Option<T>`.

Rows that have the `Option::None` value for such an optional column cannot filter on that property,
but there is no other consequence beyond that.

> ❓ Why can do keys have to be unique and non-optional?

Within `venndb` keys are meant to be able to look up,
a row which was previously received via filters.

As such it makes no sense for such keys to be:

- duplicate: it would mean: as that can result in multiple rows or the wrong row to be returned;
- optional: as that would mean the row cannot be looked up when the key is not defined;

> ❓ How can I allow some rows to match for _any_ value of a certain (filter) column?

Filter maps can allow to have a value to match all other values. It is up to you to declare the filter as such,
and to also define for that type what the _one_ value to rule them all is.

Usage:

```rust,ignore
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

## Example

Here follows an example demonstrating all the features of `VennDB`.

If you prefer a summary of what is generated, or do not understand something from the example below,
you can also read [the "Generated Code Summary" chapter](#generated-code-summary) below.

```rust
use itertools::Itertools;
use venndb::VennDB;

#[derive(Debug, VennDB)]
// These attributes are optional,
// e.g. by default the database would be called `EmployeeDB` (name + 'DB').
#[venndb(name = "EmployeeInMemDB")]
pub struct Employee {
    // you can use the `key` arg to be able to get an `Employee` instance
    // directly by this key. It will effectively establishing a mapping from key to a reference
    // of that Employee in the database. As such keys have to have unique values,
    // or else you get an error while appending / creating the DB.
    //
    // NOTE: keys do not only have to be unique, they also have to implement `Clone`!!
    //
    // A property cannot be a filter and a key at the same time,
    // trying to do so will result in a compile-team failure.
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    // filter (booleans) can be made optional,
    // meaning that the row will not be able to be filtered (found)
    // on this column when the row has a `None` value for it
    is_active: Option<bool>,
    // booleans are automatically turned into (query) filters,
    // use the `skip` arg to stop this. As such it is only really needed for
    // bool properties :)
    #[venndb(skip)]
    foo: bool,
    // non-bool values can also be turned into filters, turning them into 2D filters.
    // For each uniquely inserted Department variant that is inserted,
    // a new filter is kept track of. This allows you to apply a (query) filter
    // based on department, a pretty useful thing to be able to do.
    //
    // NOTE: this does mean that such filter-map types have to also be:
    // `PartialEq + Eq + Hash + Clone`!!
    //
    // A property cannot be a filter and a key at the same time,
    // trying to do so will result in a compile-team failure.
    #[venndb(filter)]
    department: Department,
    // similar to regular bool filters,
    // filter maps can also be optional.
    // When a filter map is optional and the row's property for that filter is None,
    // it will not be registered and thus not be able to filtered (found) on that property
    #[venndb(filter)]
    country: Option<String>,
}

fn main() {
    let db = EmployeeInMemDB::from_iter([
        RawCsvRow("1,John Doe,true,false,true,false,Engineering,USA"),
        RawCsvRow("2,Jane Doe,false,true,true,true,Sales,"),
        RawCsvRow("3,John Smith,false,false,,false,Marketing,"),
        RawCsvRow("4,Jane Smith,true,true,false,true,HR,"),
        RawCsvRow("5,John Johnson,true,true,true,true,Engineering,"),
        RawCsvRow("6,Jane Johnson,false,false,,false,Sales,BE"),
        RawCsvRow("7,John Brown,true,false,true,false,Marketing,BE"),
        RawCsvRow("8,Jane Brown,false,true,true,true,HR,BR"),
    ])
    .expect("MemDB created without errors (e.g. no duplicate keys)");

    println!(">>> Printing all employees...");
    let all_employees: Vec<_> = db.iter().collect();
    assert_eq!(all_employees.len(), 8);
    println!("All employees: {:#?}", all_employees);

    println!(">>> You can lookup an employee by any registered key...");
    let employee = db
        .get_by_id(&2)
        .expect("to have found an employee with ID 2");
    assert_eq!(employee.name, "Jane Doe");

    println!(">>> Querying for all managers...");
    let mut query = db.query();
    query.is_manager(true);
    let managers: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(managers.len(), 4);
    assert_eq!(
        managers.iter().map(|e| e.id).sorted().collect::<Vec<_>>(),
        [1, 4, 5, 7]
    );

    println!(">>> Querying for all managers with a last name of 'Johnson'...");
    let managers_result = query
        .execute()
        .expect("to have found at least one")
        .filter(|e| e.name.ends_with("Johnson"))
        .expect("to have found a manager with a last name of Johnson");
    let managers = managers_result.iter().collect::<Vec<_>>();
    assert_eq!(managers.len(), 1);
    assert_eq!(managers.iter().map(|e| e.id).collect::<Vec<_>>(), [5]);

    println!(">>> You can also just get the first result if that is all you care about...");
    let manager = managers_result.first();
    assert_eq!(manager.id, 5);

    println!(">>> Querying for a random active manager in the Engineering department...");
    let manager = query
        .reset()
        .is_active(true)
        .is_manager(true)
        .department(Department::Engineering)
        .execute()
        .expect("to have found at least one")
        .any();
    assert!(manager.id == 1 || manager.id == 5);

    println!(">>> Optional bool filters have three possible values, where None != false. An important distinction to make...");
    let mut query = db.query();
    query.is_active(false);
    let inactive_employees: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(inactive_employees.len(), 1);
    assert_eq!(inactive_employees[0].id, 4);

    println!(">>> If you want you can also get the Employees back as a Vec, dropping the DB data all together...");
    let employees = db.into_rows();
    assert_eq!(employees.len(), 8);
    assert!(employees[1].foo);
    println!("All employees: {:?}", employees);

    println!(">>> You can also get the DB back from the Vec, if you want start to query again...");
    // of course better to just keep it as a DB to begin with, but let's pretend this is ok in this example
    let mut db = EmployeeInMemDB::from_rows(employees).expect("DB created without errors");
    assert_eq!(db.iter().count(), 8);

    println!(">>> Querying for all active employees in the Sales department...");
    let mut query = db.query();
    query.is_active(true);
    query.department(Department::Sales);
    let sales_employees: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(sales_employees.len(), 1);
    assert_eq!(sales_employees[0].name, "Jane Doe");

    println!(">>> Filter maps that are optional work as well, e.g. you can query for all employees from USA...");
    query.reset().country("USA".to_owned());
    let usa_employees: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(usa_employees.len(), 1);
    assert_eq!(usa_employees[0].id, 1);

    println!(">>> At any time you can also append new employees to the DB...");
    assert!(db
        .append(RawCsvRow("8,John Doe,true,false,true,false,Engineering,"))
        .is_err());
    println!(">>> This will fail however if a property is not correct (e.g. ID (key) is not unique in this case), let's try this again...");
    assert!(db
        .append(RawCsvRow("9,John Doe,false,true,true,false,Engineering,"))
        .is_ok());
    assert_eq!(db.len(), 9);

    println!(">>> This new employee can now also be queried for...");
    let mut query = db.query();
    query.department(Department::Engineering).is_manager(false);
    let new_employee: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(new_employee.len(), 1);
    assert_eq!(new_employee[0].id, 9);

    println!(">>> You can also extend it using an IntoIterator...");
    db.extend([
        RawCsvRow("10,Glenn Doe,false,true,true,true,Engineering,"),
        RawCsvRow("11,Peter Miss,true,true,true,true,HR,USA"),
    ])
    .unwrap();
    let mut query = db.query();
    query
        .department(Department::HR)
        .is_manager(true)
        .is_active(true)
        .is_admin(true);
    let employees: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].id, 11);

    println!(">>> There are now 2 employees from USA...");
    query.reset().country("USA".to_owned());
    let employees: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(employees.len(), 2);
    assert_eq!(
        employees.iter().map(|e| e.id).sorted().collect::<Vec<_>>(),
        [1, 11]
    );

    println!(">>> All previously data is still there as well of course...");
    query
        .reset()
        .is_active(true)
        .is_manager(true)
        .department(Department::Engineering);
    let managers: Vec<_> = query
        .execute()
        .expect("to have found at least one")
        .iter()
        .collect();
    assert_eq!(managers.len(), 2);
    assert_eq!(
        managers.iter().map(|e| e.id).sorted().collect::<Vec<_>>(),
        [1, 5]
    );
}

#[derive(Debug)]
struct RawCsvRow<S>(S);

impl<S> From<RawCsvRow<S>> for Employee
where
    S: AsRef<str>,
{
    fn from(RawCsvRow(s): RawCsvRow<S>) -> Employee {
        let mut parts = s.as_ref().split(',');
        let id = parts.next().unwrap().parse().unwrap();
        let name = parts.next().unwrap().to_string();
        let is_manager = parts.next().unwrap().parse().unwrap();
        let is_admin = parts.next().unwrap().parse().unwrap();
        let is_active = match parts.next().unwrap() {
            "" => None,
            s => Some(s.parse().unwrap()),
        };
        let foo = parts.next().unwrap().parse().unwrap();
        let department = parts.next().unwrap().parse().unwrap();
        let country = match parts.next().unwrap() {
            "" => None,
            s => Some(s.to_string()),
        };
        Employee {
            id,
            name,
            is_manager,
            is_admin,
            is_active,
            foo,
            department,
            country,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Department {
    Engineering,
    Sales,
    Marketing,
    HR,
}

impl std::str::FromStr for Department {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Engineering" => Ok(Department::Engineering),
            "Sales" => Ok(Department::Sales),
            "Marketing" => Ok(Department::Marketing),
            "HR" => Ok(Department::HR),
            _ => Err(()),
        }
    }
}
```

### Generated Code Summary

In this chapter we'll list the API as generated by `VennDB` for the following example code from above:

```rust,ignore
#[derive(Debug, VennDB)]
#[venndb(name = "EmployeeInMemDB")]
pub struct Employee {
    #[venndb(key)]
    id: u32,
    name: String,
    is_manager: bool,
    is_admin: bool,
    is_active: Option<bool>,
    #[venndb(skip)]
    foo: bool,
    #[venndb(filter)]
    department: Department,
    country: Option<String>,
}
```

The following public-API datastructures will be generated:

- `struct EmployeeInMemDB`: the database, that can be used to query (by filters) or look up data (by keys);
- `enum EmployeeInMemDBError`: the error type that is returned when mutating the DB and a property of the to be inserted row;
- `enum EmployeeInMemDBErrorKind`: the kind of error that can happen as described for `EmployeeInMemDBError`;
- `struct EmployeeInMemDBQuery`: the query builder that is used to build a query that can be `execute`d to query data from the db using filters;
- `struct EmployeeInMemDBQueryResult`: the result when querying using `EmployeeInMemDBQuery` and at least one row was found that matched the defined filters;
- `struct EmployeeInMemDBQueryResultIter`: the iterator type that is used when calling `EmployeeInMemDBQueryResult::iter`. It has no methods/api other then the fact that it is an `Iterator` and can be used as one;

The visual specifiers of these datastructures will be the same as the `struct` that the `VennDB` macro is applied to.
E.g. in this example `Employee` has a specifier of `pub` so the above datastructures and their public-apy methods will also be `pub`.

There are also some other helper datastructures generated — all prefixed with the database name, e.g. `EmployeeInMemDB` in this example —
but we do not mention here as they should not be relied upon and given the prefix it should cause no conflict.
In case you do not want to expose these structures to the outside you can wrap your `struct` within its own `mod` (module).

#### Generated Code Summary: Method API

Database: (e.g. `EmployeeInMemDB`):

| fn signature | description |
| - | - |
| `EmployeeInMemDB::new() -> EmployeeInMemDB` | create a new database with zero capacity |
| `EmployeeInMemDB::default() -> EmployeeInMemDB` | same as `EmployeeInMemDB::new() -> EmployeeInMemDB` |
| `EmployeeInMemDB::capacity(capacity: usize) -> EmployeeInMemDB` | create a new database with the given capacity, but no rows already inserted |
| `EmployeeInMemDB::from_rows(rows: ::std::vec::Vec<Employee>) -> EmployeeInMemDB` or `EmployeeInMemDB::from_rows(rows: ::std::vec::Vec<Employee>) -> Result<EmployeeInMemDB, EmployeeInMemDBError<::std::vec::Vec<Employee>>>` | constructor to create the database directly from a heap-allocated list of data instances. The second version is the one used if at least one `#[venndb(key)]` property is defined, otherwise it is the first one (without the `Result`). |
| `EmployeeInMemDB::from_iter(iter: impl ::std::iter::IntoIterator<Item = impl ::std::convert::Into<Employee>>) -> EmployeeInMemDB` or `EmployeeInMemDB::from_rows(iter: impl ::std::iter::IntoIterator<Item = impl ::std::convert::Into<Employee>>) -> Result<EmployeeInMemDB, EmployeeInMemDBError<::std::vec::Vec<Employee>>>` | Same as `from_rows` but using an iterator instead. The items do not have to be an `Employee` but can be anything that can be turned into one. E.g. in our example above we defined a struct `RawCsvRow` that was turned on the fly into an `Employee`. This happens all at once prior to inserting the database, which is why the version with a result does return a `Vec` and not an iterator. |
| `EmployeeInMemDB::append(&mut self, data: impl ::std::convert::Into<Employee>)` or `EmployeeInMemDB::append(&mut self, data: impl ::std::convert::Into<Employee>) -> Result<(), EmployeeInMemDBError<Employee>>` | append a single row to the database. Depending on whether or not a `#[venndb(key)]` property is defined it will generate the `Result` version or not. Same as `from_rows` and `from_iter` |
| `EmployeeInMemDB::extend<I, Item>(&mut self, iter: I) where I: ::std::iter::IntoIterator<Item = Item>, Item: ::std::convert::Into<Employee>` or `EmployeeInMemDB::extend<I, Item>(&mut self, iter: I) -> Result<(), EmployeeInMemDBError<(Employee, I::IntoIter)>> where I: ::std::iter::IntoIterator<Item = Item>, Item: ::std::convert::Into<Employee>` | extend the database with the given iterator, once again returning a result in case such insertion can go wrong (e.g. because keys are used (duplication)). Otherwise this function will return nothing. |
| `EmployeeInMemDB::get_by_id<Q>(&self, data: impl ::std::convert::Into<Employee>) -> Option<&Employee> where Employee ::std::borrow::Borrow<Q>, Q: ::std::hash::Hash + ::std::cmp::Eq + ?::std::marker::Sized` | look up a row by the `id` key property. This method will be generated for each property marked with `#[venndb(key)`. e.g. if you have key property named `foo: MyType` property there will be also a `get_by_foo(&self, ...)` method generated. |
| `EmployeeInMemDB::query(&self) -> EmployeeInMemDBQuery` | create a `EmployeeInMemDBQuery` builder to compose a filter composition to query the database. The default builder will match all rows. See the method API for `EmployeeInMemDBQuery` for more information |

Query (e.g. `EmployeeInMemDBQuery`)

| fn signature | description |
| - | - |
| `EmployeeInMemDBQuery::reset(&mut self) -> &mut Self` | reset the query, bringing it back to the clean state it has on creation |
| `EmployeeInMemDBQuery::execute(&self) -> Option<EmployeeInMemDBQueryResult<'a>>` | return the result of the query using the set filters. It will be `None` in case no rows matched the defined filters. Or put otherwise, the result will contain at least one row when `Some(_)` is returned. |
| `EmployeeInMemDBQuery::is_manager(&mut self, value: bool) -> &mut Self` | a filter setter for a `bool` filter. One such method per `bool` filter (that isn't `skip`ped) will be available. E.g. if you have ` foo` filter then there will be a `EmployeeInMemDBQuery:foo` method. For _bool_ filters that are optional (`Option<bool>`) this method is also generated just the same. |
| `EmployeeInMemDBQuery::department(&mut self, value: Department) -> &mut Self` | a filter (map) setter for a non-`bool` filter. One such method per non-`bool` filter will be available. You can also `skip` these, but that's of course a bit pointless. The type will be equal to the actual field type. And the name will once again be equal to the original field name. Filter maps that have a `Option<T>` type have exactly the same signature. |

Query Result (e.g. `EmployeeInMemDBQueryResult`)

| fn signature | description |
| - | - |
| `EmployeeInMemDBQueryResult::first(&self) -> &Employee` | return a reference to the first matched employee found. An implementation detail is that this will be the matched row that was first inserted, but for compatibility reasons you best not rely on this if you do not have to. |
| `EmployeeInMemDBQueryResult::any(&self) -> &Employee` | return a reference to a randomly selected matched employee. The randomness can be relied upon to be fair.  |
| `EmployeeInMemDBQueryResult::iter(&self) -> `EmployeeInMemDBQueryResultIter` | return an iterator for the query result, which will allow you to iterate over all found results, and as such also collect them into an owned data structure should you wish. |
| `EmployeeInMemDBQueryResult::filter<F>(&self, predicate: F) -> Option<#EmployeeInMemDBQueryResult> where F: Fn(&#name) -> bool` | return `Some(_)` `EmployeeInMemDBQueryResult` with the same reference data, but containing (and owning) only the indexes for which the linked row matches arcoding to the given `Fn` predicate |

## ⛨ | Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## 🦀 | Compatibility

venndb is developed mostly on MacOS M-Series machines and run in production
on a variety of Linux systems. Windows support is not officially guaranteed,
but is [tested using Github Actions](https://github.com/plabayo/venndb/blob/main/.github/workflows/CI.yml) with success.

| platform | tested | test platform |
|----------|--------|---------------|
| MacOS    | ✅     | M2 (developer laptop) and macos-12 Intel ([GitHub Action](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners/about-github-hosted-runners)) |
| Windows  | ✅     | Windows 2022 ([GitHub Action](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners/about-github-hosted-runners)) |
| Linux    | ✅     | Ubuntu 22.04 ([GitHub Action](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners/about-github-hosted-runners)) |

Please [open a ticket](https://github.com/plabayo/venndb/issues) in case you have compatibility issues for your setup/platform.
Our goal is not to support all possible platformns in the world, but we do want to
support as many as we reasonably can.

### Minimum supported Rust version

venndb's MSRV is `1.75`.

[Using GitHub Actions we also test](https://github.com/plabayo/venndb/blob/main/.github/workflows/CI.yml) if `venndb` on that version still works on
the stable and beta versions of _rust_ as well.

## 🧭 | Roadmap

Please refer to <https://github.com/plabayo/venndb/milestones> to know what's on the roadmap. Is there something not on the roadmap for the next version that you would really like? Please [create a feature request](https://github.com/plabayo/venndb/issues) to request it and [become a sponsor](#--sponsors) if you can.

## 💼 | License

This project is dual-licensed under both the [MIT license][mit-license] and [Apache 2.0 License][apache-license].

## 👋 | Contributing

🎈 Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][contributing] to help you get involved in the
`venndb` project.

Contributions often come from people who already know what they want, be it a fix for a bug they encountered,
or a feature that they are missing. Please do always make a ticket if one doesn't exist already.

It's possible however that you do not yet know what specifically to contribute, and yet want to help out.
For that we thank you. You can take a look at the open issues, and in particular:

- [`good first issue`](https://github.com/plabayo/venndb/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22): issues that are good for those new to the `venndb` codebase;
- [`easy`](https://github.com/plabayo/venndb/issues?q=is%3Aissue+is%3Aopen+label%3Aeasy): issues that are seen as easy;
- [`mentor available`](https://github.com/plabayo/venndb/issues?q=is%3Aissue+is%3Aopen+label%3A%22mentor+available%22): issues for which we offer mentorship;
- [`low prio`](https://github.com/plabayo/venndb/issues?q=is%3Aissue+is%3Aopen+label%3A%22low+prio%22): low prio issues that have no immediate pressure to be finished quick, great in case you want to help out but can only do with limited time to spare;

In general, any issue not assigned already is free to be picked up by anyone else. Please do communicate in the ticket
if you are planning to pick it up, as to avoid multiple people trying to solve the same one.

Should you want to contribure this project but you do not yet know how to program in Rust, you could start learning Rust with as goal to contribute as soon as possible to `venndb` by using "[the Rust 101 Learning Guide](https://rust-lang.guide/)" as your study companion. Glen can also be hired as a mentor or teacher to give you paid 1-on-1 lessons and other similar consultancy services. You can find his contact details at <https://www.glendc.com/>.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `venndb` by you, shall be licensed as both [MIT][mit-license] and [Apache 2.0][apache-license],
without any additional terms or conditions.

[contributing]: https://github.com/plabayo/venndb/blob/main/CONTRIBUTING.md
[mit-license]: https://github.com/plabayo/venndb/blob/main/LICENSE-MIT
[apache-license]: https://github.com/plabayo/venndb/blob/main/LICENSE-APACHE

### Acknowledgements

Special thanks goes to all involved in developing, maintaining and supporting [the Rust programming language](https://www.rust-lang.org/). Also a big shoutout to the ["Write Powerful Rust Macros" book by _Sam Van Overmeire_](https://www.manning.com/books/write-powerful-rust-macros), which gave the courage to develop this crate.

Some code was also copied/forked from [google/argh](https://github.com/google/argh), for which thank you,
we are big fans of that crate. Go use it if you want to create a CLI App.

## 💖 | Sponsors

venndb is **completely free, open-source software** which needs lots of effort and time to develop and maintain.

Support this project by becoming a [sponsor][ghs-url]. One time payments are accepted [at GitHub][ghs-url] as well as at ["Buy me a Coffee"][bmac-url].

Sponsors help us continue to maintain and improve `venndb`, as well as other
Free and Open Source (FOSS) technology. It also helps us to create
educational content such as <https://github.com/plabayo/learn-rust-101>,
and other open source frameworks such as <https://github.com/plabayo/rama>.

Sponsors receive perks and depending on your regular contribution it also
allows you to rely on us for support and consulting.

Finally, you can also support us by shopping Plabayo <3 `VennDB` merchandise 🛍️ at <https://plabayo.threadless.com/>.

[![Plabayo's Store With VennDB Merchandise](https://raw.githubusercontent.com/plabayo/venndb/main/docs/img/plabayo_mech_store_venndb.png)](https://plabayo.threadless.com/)

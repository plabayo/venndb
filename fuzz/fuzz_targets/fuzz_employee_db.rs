#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use venndb::VennDB;

#[derive(Clone, Debug, Arbitrary, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u16,
    _name: String,
    earth: bool,
    #[venndb(filter)]
    faction: Faction,
}

#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, Hash)]
pub enum Faction {
    Rebel,
    Empire,
}

fuzz_target!(|rows: Vec<Employee>| {
    let _ = EmployeeDB::from_rows(rows);
});

#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use venndb::{Any, VennDB};

#[derive(Clone, Debug, Arbitrary, VennDB)]
pub struct Employee {
    #[venndb(key)]
    id: u16,
    _name: String,
    earth: bool,
    alive: Option<bool>,
    #[venndb(filter)]
    faction: Faction,
    #[venndb(filter, any)]
    planet: Option<Planet>,
}

#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, Hash)]
pub enum Faction {
    Rebel,
    Empire,
}

#[derive(Clone, Debug, Arbitrary, PartialEq, Eq, Hash)]
pub enum Planet {
    Any,
    Earth,
    Mars,
}

impl Any for Planet {
    fn is_any(&self) -> bool {
        self == &Planet::Any
    }
}

fuzz_target!(|rows: Vec<Employee>| {
    let _ = EmployeeDB::from_rows(rows);
});

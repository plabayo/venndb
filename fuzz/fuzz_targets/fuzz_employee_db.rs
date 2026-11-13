#![no_main]

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use venndb::{Any, Filter, VennDB};

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
    #[venndb(filter, set)]
    ranks: Vec<u8>,
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

/// A query, and what a row has to look like to match it.
#[derive(Clone, Debug, Arbitrary)]
pub struct Query {
    earth: Option<bool>,
    alive: Option<bool>,
    factions: Vec<Faction>,
    planets: Vec<Planet>,
    ranks: Vec<u8>,
    rank_below: Option<u8>,
    busy: Option<bool>,
    odd: Option<bool>,
}

impl Query {
    fn matches(&self, index: usize, row: &Employee, busy: &[bool]) -> bool {
        let planet = |asked: &Planet| match &row.planet {
            None => false,
            Some(Planet::Any) => true,
            Some(planet) => planet == asked,
        };
        self.earth.is_none_or(|earth| earth == row.earth)
            && self.alive.is_none_or(|alive| Some(alive) == row.alive)
            && (self.factions.is_empty() || self.factions.contains(&row.faction))
            && (self.planets.is_empty() || self.planets.iter().any(planet))
            && (self.ranks.is_empty() || self.ranks.iter().any(|r| row.ranks.contains(r)))
            && self.rank_below.is_none_or(|below| row.ranks.iter().any(|r| *r < below))
            && self.busy.is_none_or(|value| busy.get(index).copied().unwrap_or(false) == value)
            && self.odd.is_none_or(|odd| (row.id % 2 == 1) == odd)
    }
}

fuzz_target!(|input: (Vec<Employee>, Vec<Query>, Vec<bool>, u8)| {
    let (rows, queries, busy, later) = input;
    // some rows are appended after the columns are made
    let later = rows.len().saturating_sub(later as usize);
    let Ok(mut db) = EmployeeDB::from_rows(rows[..later].to_vec()) else {
        return;
    };
    let odd_column = db.column(|row| row.id % 2 == 1);
    let busy_column = db.runtime_column();
    for (index, busy) in busy.iter().enumerate().take(busy_column.len()) {
        busy_column.set(index, *busy);
    }
    if db.extend(rows[later..].to_vec()).is_err() {
        return;
    }
    let busy = &busy[..busy.len().min(busy_column.len())];

    for query in queries {
        let mut q = db.query();
        if let Some(earth) = query.earth {
            q.earth(earth);
        }
        if let Some(alive) = query.alive {
            q.alive(alive);
        }
        for faction in &query.factions {
            q.faction(faction.clone());
        }
        for planet in &query.planets {
            q.planet(planet.clone());
        }
        for rank in &query.ranks {
            q.ranks(*rank);
        }
        if let Some(below) = query.rank_below {
            q.ranks_where(|rank| *rank < below);
        }
        if let Some(value) = query.busy {
            q.and(Filter::Runtime(&busy_column, value));
        }
        if let Some(odd) = query.odd {
            // rows appended later are not in the column: only filter on those in it
            if odd {
                q.and(Filter::Is(&odd_column));
            } else {
                q.and(Filter::Not(&odd_column));
            }
        }

        let expected: Vec<u16> = db
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                let mut query = query.clone();
                if *index >= later {
                    // appended later: in no column, so never odd
                    if query.odd == Some(true) {
                        return false;
                    }
                    query.odd = None;
                }
                query.matches(*index, row, busy)
            })
            .map(|(_, row)| row.id)
            .collect();

        match q.execute() {
            None => assert!(expected.is_empty(), "{query:?}"),
            Some(result) => {
                let found: Vec<u16> = result.iter().map(|row| row.id).collect();
                assert_eq!(found, expected, "{query:?}");
                assert_eq!(result.len(), expected.len());
                assert_eq!(result.first().id, expected[0]);
                assert!(expected.contains(&result.any().id));
                let even = result.any_where(|row| row.id % 2 == 0);
                assert_eq!(even.is_some(), expected.iter().any(|id| id % 2 == 0));
                assert!(even.is_none_or(|row| row.id % 2 == 0 && expected.contains(&row.id)));
            }
        }
    }
});

//! A fleet of servers, to show what a database of rows with
//! richer fields looks like: enums with data, sets of them,
//! wildcards and columns that change while the database is in use.
//!
//! Run with `cargo run --example fleet`.

use std::collections::HashSet;
use venndb::{Filter, VennDB};

/// A way to reach a server. One server offers several of these.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Http { version: HttpVersion, tls: bool },
    Ssh { forwarding: bool, sftp: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpVersion {
    H1,
    H2,
    H3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    Edge,
    Metal,
    Cloud,
}

/// A location, `*` is a server that can be used for any location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location(String);

impl Location {
    fn is_anywhere(&self) -> bool {
        self.0 == "*"
    }
}

impl From<&str> for Location {
    fn from(name: &str) -> Self {
        Self(name.trim().to_lowercase())
    }
}

#[derive(Debug, VennDB)]
#[venndb(name = "Fleet")]
pub struct Server {
    #[venndb(key)]
    id: u32,
    address: String,
    /// A plain enum is a filter like any other value.
    #[venndb(filter)]
    tier: Tier,
    /// A row matches each of the values in a set.
    #[venndb(filter, set)]
    endpoints: HashSet<Endpoint>,
    /// No need to implement `venndb::Any`, a function will do.
    #[venndb(filter, any = Location::is_anywhere)]
    country: Option<Location>,
    #[venndb(filter, any = Location::is_anywhere)]
    city: Option<Location>,
    /// Bools are filters without asking.
    gpu: bool,
}

fn server(id: u32, tier: Tier, endpoints: &[Endpoint], country: &str, city: &str) -> Server {
    Server {
        id,
        address: format!("10.0.0.{id}"),
        tier,
        endpoints: endpoints.iter().cloned().collect(),
        country: Some(country.into()),
        city: (!city.is_empty()).then(|| city.into()),
        gpu: id.is_multiple_of(2),
    }
}

fn fleet() -> Fleet {
    use Endpoint::{Http, Ssh};
    use HttpVersion::{H1, H2, H3};

    let h2 = Http {
        version: H2,
        tls: true,
    };
    let h3 = Http {
        version: H3,
        tls: true,
    };
    let plain = Http {
        version: H1,
        tls: false,
    };
    let ssh = Ssh {
        forwarding: false,
        sftp: true,
    };
    let jump = Ssh {
        forwarding: true,
        sftp: false,
    };

    Fleet::from_rows(vec![
        server(1, Tier::Edge, &[h2.clone(), h3.clone()], "BE", "Ghent"),
        server(2, Tier::Edge, &[h2], "BE", "Antwerp"),
        server(3, Tier::Metal, &[plain, ssh.clone()], "NL", "Utrecht"),
        server(4, Tier::Metal, &[ssh, jump.clone()], "BE", "Ghent"),
        server(5, Tier::Cloud, &[h3, jump], "*", "*"),
        server(6, Tier::Cloud, &[], "FR", ""),
    ])
    .expect("unique ids")
}

fn ids<'a>(servers: impl IntoIterator<Item = &'a Server>) -> Vec<u32> {
    servers.into_iter().map(|server| server.id).collect()
}

fn main() {
    let fleet = fleet();

    // plain values: `AND` between fields, `OR` within one
    let mut query = fleet.query();
    query.tier(Tier::Edge).tier(Tier::Metal).country("be");
    assert_eq!(ids(query.execute().unwrap().iter()), [1, 2, 4]);

    // a wildcard row matches any value asked for
    let mut query = fleet.query();
    query.country("FR");
    assert_eq!(ids(query.execute().unwrap().iter()), [5, 6]);
    // but a row without a value matches none
    query.city("Paris");
    assert_eq!(ids(query.execute().unwrap().iter()), [5]);

    // a set matches on each of its values
    let mut query = fleet.query();
    query.endpoints(Endpoint::Http {
        version: HttpVersion::H2,
        tls: true,
    });
    assert_eq!(ids(query.execute().unwrap().iter()), [1, 2]);

    // or on whatever its values look like: the predicate is called
    // once per distinct endpoint in the fleet, not once per server
    let mut query = fleet.query();
    query
        .endpoints_where(|endpoint| matches!(endpoint, Endpoint::Http { tls: true, .. }))
        .gpu(true);
    assert_eq!(ids(query.execute().unwrap().iter()), [2]);

    let mut query = fleet.query();
    query.endpoints_where(|endpoint| {
        matches!(
            endpoint,
            Endpoint::Ssh {
                forwarding: true,
                ..
            }
        )
    });
    assert_eq!(ids(query.execute().unwrap().iter()), [4, 5]);

    // a fact about a row that is no field: computed once, used like a column
    let redundant = fleet.column(|server| server.endpoints.len() > 1);
    // state that changes while the fleet is in use: set from any thread, no lock
    let draining = fleet.runtime_column();
    draining.set(fleet.index_by_id(&4).unwrap(), true);

    let mut query = fleet.query();
    query
        .country("BE")
        .and(Filter::Is(&redundant))
        .and(Filter::Runtime(&draining, false));
    assert_eq!(ids(query.execute().unwrap().iter()), [1, 5]);

    // server 4 is back, the same query now finds it
    draining.set(fleet.index_by_id(&4).unwrap(), false);
    let found = query.execute().unwrap();
    assert_eq!(found.len(), 3);

    // a random server, and one for a condition that fits no column
    let lucky = found.any();
    let odd = found.any_where(|server| server.id % 2 == 1).unwrap();
    println!(
        "picked server {} at {} and odd server {}",
        lucky.id, lucky.address, odd.id
    );
}

#[test]
fn runs() {
    main();
}

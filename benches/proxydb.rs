//! Benchmarks of a proxy database shaped like the one of `rama`:
//! 7 bool columns and 5 label columns, of which 3 with many values.
//!
//! Run with `just bench`, or `just bench-full` to include 1 000 000 rows.

mod proxies;

use divan::{AllocProfiler, Bencher};
use proxies::{
    NaiveProxyDB, PredicateProxyDB, Proxy, ProxyDB, Query, RoaringProxyDB, Scenario, SqliteProxyDB,
    VennProxyDB,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    verify();
    divan::main();
}

const SIZES: &[usize] = if cfg!(feature = "bench-full") {
    &[1_000, 100_000, 1_000_000]
} else {
    &[1_000, 100_000]
};

const QUERIES: usize = 512;

static DBS: Mutex<Option<HashMap<(TypeId, usize), Arc<dyn Any + Send + Sync>>>> = Mutex::new(None);
static ROUND: AtomicUsize = AtomicUsize::new(0);

/// The database of the given type and size, created once.
fn db<D: ProxyDB>(n: usize) -> Arc<D> {
    let mut dbs = DBS.lock().unwrap();
    dbs.get_or_insert_default()
        .entry((TypeId::of::<D>(), n))
        .or_insert_with(|| Arc::new(D::create(proxies::rows(n))))
        .clone()
        .downcast()
        .unwrap()
}

fn next(queries: &[Query]) -> &Query {
    &queries[ROUND.fetch_add(1, Ordering::Relaxed) % queries.len()]
}

/// All implementations have to agree before they are compared.
fn verify() {
    let rows = proxies::rows(2_000);
    let naive = NaiveProxyDB::create(rows.clone());
    let venn = VennProxyDB::create(rows.clone());
    let roaring = RoaringProxyDB::create(rows.clone());
    let sqlite = SqliteProxyDB::create(rows.clone());
    for scenario in SCENARIOS {
        for query in proxies::queries(&rows, scenario, 64) {
            let expected = naive.count(&query);
            assert_eq!(expected, venn.count(&query), "venndb: {scenario:?}");
            assert_eq!(expected, roaring.count(&query), "roaring: {scenario:?}");
            assert_eq!(expected, sqlite.count(&query), "sqlite: {scenario:?}");
        }
    }
}

const SCENARIOS: [Scenario; 7] = [
    Scenario::Bools,
    Scenario::Country,
    Scenario::PoolCountry,
    Scenario::City,
    Scenario::Asn,
    Scenario::Countries,
    Scenario::Cities,
];

fn pick<D: ProxyDB>(bencher: Bencher, n: usize, scenario: Scenario) {
    let db = db::<D>(n);
    let queries = proxies::queries(&proxies::rows(n), scenario, QUERIES);
    bencher.bench(|| db.pick(next(&queries)));
}

/// Keeps 9 out of 10 proxies, as a filter on blocked proxies would.
fn predicate(proxy: &Proxy) -> bool {
    (proxy.id.wrapping_mul(2_654_435_761) >> 16) % 10 != 3
}

fn pick_if<D: PredicateProxyDB>(bencher: Bencher, n: usize, scenario: Scenario) {
    let db = db::<D>(n);
    let queries = proxies::queries(&proxies::rows(n), scenario, QUERIES);
    bencher.bench(|| db.pick_if(next(&queries), predicate));
}

/// Time and peak memory to create the database.
#[divan::bench(
    types = [VennProxyDB, NaiveProxyDB, RoaringProxyDB, SqliteProxyDB],
    args = SIZES,
    sample_count = 3,
    sample_size = 1,
)]
fn build<D: ProxyDB>(bencher: Bencher, n: usize) {
    bencher
        .with_inputs(|| proxies::rows(n))
        .bench_values(D::create);
}

#[divan::bench(types = [VennProxyDB, NaiveProxyDB, RoaringProxyDB, SqliteProxyDB], args = SIZES)]
fn get_by_id<D: ProxyDB>(bencher: Bencher, n: usize) {
    let db = db::<D>(n);
    bencher.bench(|| db.get((ROUND.fetch_add(1, Ordering::Relaxed) % n) as u64));
}

macro_rules! scenarios {
    ($($name:ident => $scenario:expr,)+) => {
        mod pick {
            use super::*;
            $(
                #[divan::bench(
                    types = [VennProxyDB, NaiveProxyDB, RoaringProxyDB, SqliteProxyDB],
                    args = SIZES,
                )]
                fn $name<D: ProxyDB>(bencher: Bencher, n: usize) {
                    pick::<D>(bencher, n, $scenario);
                }
            )+
        }

        mod pick_if {
            use super::*;
            $(
                #[divan::bench(
                    types = [VennProxyDB, NaiveProxyDB, RoaringProxyDB],
                    args = SIZES,
                )]
                fn $name<D: PredicateProxyDB>(bencher: Bencher, n: usize) {
                    pick_if::<D>(bencher, n, $scenario);
                }
            )+
        }
    };
}

scenarios! {
    bools => Scenario::Bools,
    country => Scenario::Country,
    pool_country => Scenario::PoolCountry,
    city => Scenario::City,
    asn => Scenario::Asn,
    countries => Scenario::Countries,
    cities => Scenario::Cities,
}

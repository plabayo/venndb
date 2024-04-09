mod proxies;
use divan::AllocProfiler;
use proxies::{InMemProxyDB, NaiveProxyDB, ProxyDB, SqlLiteProxyDB};
use std::sync::atomic::AtomicUsize;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const POOLS: [&str; 14] = [
    "poolA", "poolB", "poolC", "poolD", "poolE", "poolF", "poolG", "poolH", "poolI", "poolJ",
    "poolA", "poolB", "poolC", "poolD",
];
const COUNTRIES: [&str; 13] = [
    "US", "CA", "GB", "DE", "FR", "IT", "ES", "AU", "JP", "CN", "FR", "IT", "ES",
];

const COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_round() -> usize {
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

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

#[divan::bench]
fn venn_proxy_db_100(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| InMemProxyDB::create(100))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn naive_proxy_db_100(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| NaiveProxyDB::create(100))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn sql_lite_proxy_db_100(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| SqlLiteProxyDB::create(100))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn venn_proxy_db_12_500(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| InMemProxyDB::create(12_500))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn naive_proxy_db_12_500(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| NaiveProxyDB::create(12_500))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn sql_lite_proxy_db_12_500(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| SqlLiteProxyDB::create(12_500))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn venn_proxy_db_100_000(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| InMemProxyDB::create(100_000))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn naive_proxy_db_100_000(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| NaiveProxyDB::create(100_000))
        .bench_refs(|db| test_db(db));
}

#[divan::bench]
fn sql_lite_proxy_db_100_000(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| SqlLiteProxyDB::create(100_000))
        .bench_refs(|db| test_db(db));
}

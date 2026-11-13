//! A proxy database shaped like the one of `rama`, implemented four ways.

use roaring::RoaringBitmap;
use std::collections::HashMap;
use venndb::{Any, VennDB};

#[derive(Debug, Clone, VennDB)]
#[venndb(name = "VennProxyDB")]
pub(super) struct Proxy {
    #[venndb(key)]
    pub(super) id: u64,
    pub(super) address: String,
    pub(super) tcp: bool,
    pub(super) udp: bool,
    pub(super) http: bool,
    pub(super) socks5: bool,
    pub(super) datacenter: bool,
    pub(super) residential: bool,
    pub(super) mobile: bool,
    #[venndb(filter, any)]
    pub(super) pool: Option<Label>,
    #[venndb(filter, any)]
    pub(super) country: Option<Label>,
    #[venndb(filter, any)]
    pub(super) state: Option<Label>,
    #[venndb(filter, any)]
    pub(super) city: Option<Label>,
    #[venndb(filter, any)]
    pub(super) asn: Option<Label>,
}

pub(super) const BOOLS: usize = 7;
pub(super) const LABELS: usize = 5;
const BOOL_NAMES: [&str; BOOLS] = [
    "tcp",
    "udp",
    "http",
    "socks5",
    "datacenter",
    "residential",
    "mobile",
];
const LABEL_NAMES: [&str; LABELS] = ["pool", "country", "state", "city", "asn"];

impl Proxy {
    fn bools(&self) -> [bool; BOOLS] {
        [
            self.tcp,
            self.udp,
            self.http,
            self.socks5,
            self.datacenter,
            self.residential,
            self.mobile,
        ]
    }

    pub(super) fn labels(&self) -> [Option<&Label>; LABELS] {
        [
            self.pool.as_ref(),
            self.country.as_ref(),
            self.state.as_ref(),
            self.city.as_ref(),
            self.asn.as_ref(),
        ]
    }
}

/// A normalized label, `*` matches any value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Label(String);

impl Label {
    fn new(kind: &str, n: u32) -> Self {
        Self(format!("{kind}{n}"))
    }

    fn any() -> Self {
        Self("*".to_owned())
    }
}

impl Any for Label {
    fn is_any(&self) -> bool {
        self.0 == "*"
    }
}

/// A conjunction of filters: `AND` between columns, `OR` within one.
#[derive(Debug, Clone, Default)]
pub(super) struct Query {
    pub(super) bools: [Option<bool>; BOOLS],
    pub(super) labels: [Vec<Label>; LABELS],
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// Skewed towards the low numbers, as real world labels are.
    fn skewed(&mut self, cardinality: usize) -> u32 {
        let u = self.unit();
        (u * u * u * cardinality as f64) as u32
    }
}

/// Deterministic rows: 20 pools, 200 countries and up to
/// 1 000 states, 10 000 cities and 5 000 ASNs. 1% of the labels is `*`.
pub(super) fn rows(n: usize) -> Vec<Proxy> {
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    let cardinality = [
        20,
        200,
        (n / 100).clamp(5, 1_000),
        (n / 10).clamp(10, 10_000),
        (n / 20).clamp(10, 5_000),
    ];
    let missing = [0.05, 0.0, 0.3, 0.2, 0.1];
    (0..n)
        .map(|i| {
            let kind = rng.next() % 10;
            let mut labels = [const { None }; LABELS];
            for (c, label) in labels.iter_mut().enumerate() {
                *label = if rng.chance(missing[c]) {
                    None
                } else if rng.chance(0.01) {
                    Some(Label::any())
                } else {
                    Some(Label::new(LABEL_NAMES[c], rng.skewed(cardinality[c])))
                };
            }
            let [pool, country, state, city, asn] = labels;
            Proxy {
                id: i as u64,
                address: format!("10.{}.{}.{}:8080", (i >> 16) & 255, (i >> 8) & 255, i & 255),
                tcp: rng.chance(0.95),
                udp: rng.chance(0.3),
                http: rng.chance(0.7),
                socks5: rng.chance(0.5),
                datacenter: kind < 6,
                residential: (6..9).contains(&kind),
                mobile: kind == 9,
                pool,
                country,
                state,
                city,
                asn,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scenario {
    /// `tcp + socks5 + datacenter`
    Bools,
    /// `tcp + country`
    Country,
    /// `tcp + !mobile + pool + country`
    PoolCountry,
    /// `tcp + datacenter + country + city`
    City,
    /// `udp + !residential + asn`
    Asn,
    /// `mobile + pool + 3 countries`
    Countries,
    /// `tcp + 5 cities`
    Cities,
}

/// The queries of a scenario, with labels taken from existing rows.
pub(super) fn queries(rows: &[Proxy], scenario: Scenario, n: usize) -> Vec<Query> {
    let mut rng = XorShift(42);
    let mut row = || loop {
        let row = &rows[(rng.next() % rows.len() as u64) as usize];
        if row.labels().iter().all(|l| l.is_some_and(|l| !l.is_any())) {
            return row;
        }
    };
    (0..n)
        .map(|_| {
            let (a, b, c) = (row(), row(), row());
            let label = |r: &Proxy, c: usize| r.labels()[c].cloned().unwrap();
            let mut q = Query::default();
            match scenario {
                Scenario::Bools => {
                    q.bools[0] = Some(true);
                    q.bools[3] = Some(true);
                    q.bools[4] = Some(true);
                }
                Scenario::Country => {
                    q.bools[0] = Some(true);
                    q.labels[1] = vec![label(a, 1)];
                }
                Scenario::PoolCountry => {
                    q.bools[0] = Some(true);
                    q.bools[6] = Some(false);
                    q.labels[0] = vec![label(a, 0)];
                    q.labels[1] = vec![label(a, 1)];
                }
                Scenario::City => {
                    q.bools[0] = Some(true);
                    q.bools[4] = Some(a.datacenter);
                    q.labels[1] = vec![label(a, 1)];
                    q.labels[3] = vec![label(a, 3)];
                }
                Scenario::Asn => {
                    q.bools[1] = Some(true);
                    q.bools[5] = Some(false);
                    q.labels[4] = vec![label(a, 4)];
                }
                Scenario::Countries => {
                    q.bools[6] = Some(true);
                    q.labels[0] = vec![label(a, 0)];
                    q.labels[1] = vec![label(a, 1), label(b, 1), label(c, 1)];
                }
                Scenario::Cities => {
                    q.bools[0] = Some(true);
                    q.labels[3] = vec![
                        label(a, 3),
                        label(b, 3),
                        label(c, 3),
                        Label::new("city", 3),
                        Label::new("city", 7),
                    ];
                }
            }
            q
        })
        .collect()
}

pub(super) trait ProxyDB: Sized + Send + Sync + 'static {
    fn create(rows: Vec<Proxy>) -> Self;

    /// Look a proxy up by its id.
    fn get(&self, id: u64) -> Option<u64>;

    /// The id of a random proxy matching the query.
    fn pick(&self, query: &Query) -> Option<u64>;

    /// The number of proxies matching the query.
    fn count(&self, query: &Query) -> usize;
}

pub(super) trait PredicateProxyDB: ProxyDB {
    /// The id of a random proxy matching the query and the predicate.
    fn pick_if(&self, query: &Query, predicate: impl Fn(&Proxy) -> bool) -> Option<u64>;
}

impl VennProxyDB {
    fn run(&self, q: &Query) -> Option<VennProxyDBQueryResult<'_>> {
        let mut query = self.query();
        Self::apply(q, &mut query);
        query.execute()
    }

    fn apply(q: &Query, query: &mut VennProxyDBQuery<'_>) {
        let [tcp, udp, http, socks5, datacenter, residential, mobile] = q.bools;
        if let Some(v) = tcp {
            query.tcp(v);
        }
        if let Some(v) = udp {
            query.udp(v);
        }
        if let Some(v) = http {
            query.http(v);
        }
        if let Some(v) = socks5 {
            query.socks5(v);
        }
        if let Some(v) = datacenter {
            query.datacenter(v);
        }
        if let Some(v) = residential {
            query.residential(v);
        }
        if let Some(v) = mobile {
            query.mobile(v);
        }
        for v in &q.labels[0] {
            query.pool(v.clone());
        }
        for v in &q.labels[1] {
            query.country(v.clone());
        }
        for v in &q.labels[2] {
            query.state(v.clone());
        }
        for v in &q.labels[3] {
            query.city(v.clone());
        }
        for v in &q.labels[4] {
            query.asn(v.clone());
        }
    }
}

impl VennProxyDB {
    /// The id of a random proxy matching the query, that is not blocked.
    pub(super) fn pick_unblocked(
        &self,
        query: &Query,
        blocked: &venndb::RuntimeColumn,
    ) -> Option<u64> {
        let mut plan = self.query();
        plan.and(venndb::Filter::Runtime(blocked, false));
        Self::apply(query, &mut plan);
        plan.execute().map(|result| result.any().id)
    }
}

impl ProxyDB for VennProxyDB {
    fn create(rows: Vec<Proxy>) -> Self {
        Self::from_rows(rows).unwrap()
    }

    fn get(&self, id: u64) -> Option<u64> {
        self.get_by_id(&id).map(|p| p.id)
    }

    fn pick(&self, query: &Query) -> Option<u64> {
        self.run(query).map(|result| result.any().id)
    }

    fn count(&self, query: &Query) -> usize {
        self.run(query).map_or(0, |result| result.iter().count())
    }
}

impl PredicateProxyDB for VennProxyDB {
    fn pick_if(&self, query: &Query, predicate: impl Fn(&Proxy) -> bool) -> Option<u64> {
        self.run(query)?.any_where(predicate).map(|proxy| proxy.id)
    }
}

/// A linear scan, without any allocation.
#[derive(Debug)]
pub(super) struct NaiveProxyDB(Vec<Proxy>);

fn matches(proxy: &Proxy, query: &Query) -> bool {
    let bools = proxy.bools();
    let labels = proxy.labels();
    (0..BOOLS).all(|c| query.bools[c].is_none_or(|v| v == bools[c]))
        && (0..LABELS).all(|c| {
            query.labels[c].is_empty()
                || labels[c].is_some_and(|l| l.is_any() || query.labels[c].contains(l))
        })
}

impl NaiveProxyDB {
    fn sample(&self, predicate: impl Fn(&Proxy) -> bool) -> Option<u64> {
        let mut pick = None;
        for (seen, proxy) in self.0.iter().filter(|p| predicate(p)).enumerate() {
            if rand::random_range(0..=seen) == 0 {
                pick = Some(proxy.id);
            }
        }
        pick
    }
}

impl ProxyDB for NaiveProxyDB {
    fn create(rows: Vec<Proxy>) -> Self {
        Self(rows)
    }

    fn get(&self, id: u64) -> Option<u64> {
        self.0.iter().find(|p| p.id == id).map(|p| p.id)
    }

    fn pick(&self, query: &Query) -> Option<u64> {
        self.sample(|p| matches(p, query))
    }

    fn count(&self, query: &Query) -> usize {
        self.0.iter().filter(|p| matches(p, query)).count()
    }
}

impl PredicateProxyDB for NaiveProxyDB {
    fn pick_if(&self, query: &Query, predicate: impl Fn(&Proxy) -> bool) -> Option<u64> {
        self.sample(|p| matches(p, query) && predicate(p))
    }
}

/// An index of [`RoaringBitmap`]s, one per bool column and label.
#[derive(Debug)]
pub(super) struct RoaringProxyDB {
    rows: Vec<Proxy>,
    ids: HashMap<u64, u32>,
    bools: [RoaringBitmap; BOOLS],
    labels: [HashMap<Label, RoaringBitmap>; LABELS],
    any: [RoaringBitmap; LABELS],
}

impl RoaringProxyDB {
    fn run(&self, query: &Query) -> Option<RoaringBitmap> {
        let mut operands = Vec::new();
        for c in 0..LABELS {
            if query.labels[c].is_empty() {
                continue;
            }
            let mut union = self.any[c].clone();
            for label in &query.labels[c] {
                if let Some(bitmap) = self.labels[c].get(label) {
                    union |= bitmap;
                }
            }
            operands.push(union);
        }
        operands.sort_by_key(RoaringBitmap::len);
        let mut operands = operands.into_iter();
        let mut result = operands.next();
        for operand in operands {
            *result.as_mut()? &= operand;
        }
        for c in 0..BOOLS {
            let Some(value) = query.bools[c] else {
                continue;
            };
            let result = result.get_or_insert_with(|| {
                let mut all = RoaringBitmap::new();
                all.insert_range(0..self.rows.len() as u32);
                all
            });
            if value {
                *result &= &self.bools[c];
            } else {
                *result -= &self.bools[c];
            }
        }
        result.filter(|r| !r.is_empty())
    }
}

impl ProxyDB for RoaringProxyDB {
    fn create(rows: Vec<Proxy>) -> Self {
        let mut db = Self {
            ids: HashMap::with_capacity(rows.len()),
            bools: Default::default(),
            labels: Default::default(),
            any: Default::default(),
            rows: Vec::new(),
        };
        for (i, proxy) in rows.iter().enumerate() {
            let i = i as u32;
            db.ids.insert(proxy.id, i);
            for (c, value) in proxy.bools().into_iter().enumerate() {
                if value {
                    db.bools[c].insert(i);
                }
            }
            for (c, label) in proxy.labels().into_iter().enumerate() {
                match label {
                    Some(label) if label.is_any() => {
                        db.any[c].insert(i);
                    }
                    Some(label) => {
                        db.labels[c].entry(label.clone()).or_default().insert(i);
                    }
                    None => (),
                }
            }
        }
        db.rows = rows;
        db
    }

    fn get(&self, id: u64) -> Option<u64> {
        self.ids.get(&id).map(|i| self.rows[*i as usize].id)
    }

    fn pick(&self, query: &Query) -> Option<u64> {
        let result = self.run(query)?;
        let n = rand::random_range(0..result.len() as u32);
        result.select(n).map(|i| self.rows[i as usize].id)
    }

    fn count(&self, query: &Query) -> usize {
        self.run(query).map_or(0, |r| r.len() as usize)
    }
}

impl PredicateProxyDB for RoaringProxyDB {
    fn pick_if(&self, query: &Query, predicate: impl Fn(&Proxy) -> bool) -> Option<u64> {
        let found: Vec<u32> = self
            .run(query)?
            .into_iter()
            .filter(|i| predicate(&self.rows[*i as usize]))
            .collect();
        (!found.is_empty())
            .then(|| self.rows[found[rand::random_range(0..found.len())] as usize].id)
    }
}

/// An in-memory SQLite table, with an index on every label.
#[non_exhaustive]
pub(super) struct SqliteProxyDB {
    conn: std::sync::Mutex<sqlite::Connection>,
}

fn sql_text(label: Option<&Label>) -> String {
    label.map_or_else(|| "NULL".to_owned(), |l| format!("'{}'", l.0))
}

fn sql_where(query: &Query) -> String {
    let mut clauses = vec!["1 = 1".to_owned()];
    for (name, value) in BOOL_NAMES.iter().zip(query.bools) {
        if let Some(value) = value {
            clauses.push(format!("{name} = {}", value as u8));
        }
    }
    for (name, labels) in LABEL_NAMES.iter().zip(&query.labels) {
        if !labels.is_empty() {
            let values: Vec<_> = labels.iter().map(|l| sql_text(Some(l))).collect();
            clauses.push(format!("{name} IN ('*', {})", values.join(", ")));
        }
    }
    clauses.join(" AND ")
}

impl SqliteProxyDB {
    fn first(&self, statement: &str) -> Option<u64> {
        let conn = self.conn.lock().unwrap();
        let row = conn.prepare(statement).unwrap().into_iter().next()?.ok()?;
        Some(row.read::<i64, _>(0) as u64)
    }
}

impl ProxyDB for SqliteProxyDB {
    fn create(rows: Vec<Proxy>) -> Self {
        let conn = sqlite::open(":memory:").unwrap();
        conn.execute(
            "CREATE TABLE proxies (
                id INTEGER PRIMARY KEY, address TEXT NOT NULL,
                tcp BOOLEAN NOT NULL, udp BOOLEAN NOT NULL, http BOOLEAN NOT NULL,
                socks5 BOOLEAN NOT NULL, datacenter BOOLEAN NOT NULL,
                residential BOOLEAN NOT NULL, mobile BOOLEAN NOT NULL,
                pool TEXT, country TEXT, state TEXT, city TEXT, asn TEXT
            )",
        )
        .unwrap();
        conn.execute("BEGIN").unwrap();
        for proxy in &rows {
            let bools = proxy.bools().map(|b| (b as u8).to_string()).join(", ");
            let labels = proxy.labels().map(sql_text).join(", ");
            conn.execute(format!(
                "INSERT INTO proxies VALUES ({}, '{}', {bools}, {labels})",
                proxy.id, proxy.address,
            ))
            .unwrap();
        }
        conn.execute("COMMIT").unwrap();
        for name in LABEL_NAMES {
            conn.execute(format!("CREATE INDEX idx_{name} ON proxies ({name})"))
                .unwrap();
        }
        Self {
            conn: std::sync::Mutex::new(conn),
        }
    }

    fn get(&self, id: u64) -> Option<u64> {
        self.first(&format!("SELECT id FROM proxies WHERE id = {id}"))
    }

    fn pick(&self, query: &Query) -> Option<u64> {
        self.first(&format!(
            "SELECT id FROM proxies WHERE {} ORDER BY RANDOM() LIMIT 1",
            sql_where(query)
        ))
    }

    fn count(&self, query: &Query) -> usize {
        self.first(&format!(
            "SELECT COUNT(*) FROM proxies WHERE {}",
            sql_where(query)
        ))
        .unwrap_or_default() as usize
    }
}

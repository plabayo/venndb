use sqlite::Row;
use std::{borrow::Cow, ops::Deref};
use venndb::{Any, VennDB};

pub trait ProxyDB: Sized {
    fn create(n: usize) -> Self;

    fn get(&self, id: u64) -> Option<Cow<Proxy>>;
    fn any_tcp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>>;
    fn any_socks5_isp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>>;
}

#[derive(Debug, Clone, VennDB)]
#[venndb(name = "InMemProxyDB")]
pub struct Proxy {
    #[venndb(key)]
    pub id: u64,
    pub address: String,
    pub username: String,
    pub password: String,
    pub tcp: bool,
    pub udp: bool,
    pub http: bool,
    pub socks5: bool,
    pub datacenter: bool,
    pub residential: bool,
    pub mobile: bool,
    #[venndb(filter)]
    pub pool: Option<NormalizedString>,
    #[venndb(filter, any)]
    pub country: NormalizedString,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedString(String);

impl<S: AsRef<str>> From<S> for NormalizedString {
    fn from(s: S) -> Self {
        Self(s.as_ref().trim().to_lowercase())
    }
}

impl Any for NormalizedString {
    fn is_any(&self) -> bool {
        self.0 == "*"
    }
}

impl Deref for NormalizedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Proxy {
    fn from(s: String) -> Self {
        let mut parts = s.split(',');
        Self {
            id: parts.next().unwrap().parse().unwrap(),
            address: parts.next().unwrap().to_string(),
            username: parts.next().unwrap().to_string(),
            password: parts.next().unwrap().to_string(),
            tcp: parts.next().unwrap().parse().unwrap(),
            udp: parts.next().unwrap().parse().unwrap(),
            http: parts.next().unwrap().parse().unwrap(),
            socks5: parts.next().unwrap().parse().unwrap(),
            datacenter: parts.next().unwrap().parse().unwrap(),
            residential: parts.next().unwrap().parse().unwrap(),
            mobile: parts.next().unwrap().parse().unwrap(),
            pool: match parts.next().unwrap() {
                "" => None,
                s => Some(s.into()),
            },
            country: parts.next().unwrap().into(),
        }
    }
}
const RAW_PROXIES_CSV: &str = include_str!("fake_proxies.csv");

impl ProxyDB for InMemProxyDB {
    fn create(n: usize) -> Self {
        let mut db = InMemProxyDB::with_capacity(n);
        for line in RAW_PROXIES_CSV.lines().take(n) {
            db.append(Proxy::from(line.to_string())).unwrap();
        }
        db
    }

    fn get(&self, id: u64) -> Option<Cow<Proxy>> {
        self.get_by_id(&id).map(Cow::Borrowed)
    }

    fn any_tcp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let mut query = self.query();
        query.tcp(true).pool(pool.into()).country(country.into());
        query.execute().map(|result| {
            let proxy_ref = result.any();
            Cow::Borrowed(proxy_ref)
        })
    }

    fn any_socks5_isp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let mut query = self.query();
        query
            .socks5(true)
            .datacenter(true)
            .residential(true)
            .pool(pool.into())
            .country(country.into());
        query.execute().map(|result| {
            let proxy_ref = result.any();
            Cow::Borrowed(proxy_ref)
        })
    }
}

#[derive(Debug)]
pub struct NaiveProxyDB {
    proxies: Vec<Proxy>,
}

impl ProxyDB for NaiveProxyDB {
    fn create(n: usize) -> Self {
        let proxies = RAW_PROXIES_CSV
            .lines()
            .take(n)
            .map(|line| Proxy::from(line.to_string()))
            .collect();
        NaiveProxyDB { proxies }
    }

    fn get(&self, id: u64) -> Option<Cow<Proxy>> {
        self.proxies.iter().find(|p| p.id == id).map(Cow::Borrowed)
    }

    fn any_tcp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let found_proxies: Vec<_> = self
            .proxies
            .iter()
            .filter(|p| p.tcp && p.pool == Some(pool.into()) && p.country == country.into())
            .collect();
        if found_proxies.is_empty() {
            None
        } else {
            use rand::Rng;
            let index = rand::thread_rng().gen_range(0..found_proxies.len());
            Some(Cow::Borrowed(found_proxies[index]))
        }
    }

    fn any_socks5_isp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let found_proxies: Vec<_> = self
            .proxies
            .iter()
            .filter(|p| {
                p.socks5
                    && p.datacenter
                    && p.residential
                    && p.pool == Some(pool.into())
                    && p.country == country.into()
            })
            .collect();
        if found_proxies.is_empty() {
            None
        } else {
            use rand::Rng;
            let index = rand::thread_rng().gen_range(0..found_proxies.len());
            Some(Cow::Borrowed(found_proxies[index]))
        }
    }
}

#[non_exhaustive]
pub struct SqlLiteProxyDB {
    conn: sqlite::Connection,
}

impl std::fmt::Debug for SqlLiteProxyDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlLiteProxyDB").finish()
    }
}

impl ProxyDB for SqlLiteProxyDB {
    fn create(n: usize) -> Self {
        let conn = sqlite::open(":memory:").unwrap();

        // create the DB
        conn.execute(
            "CREATE TABLE proxies (
                id INTEGER PRIMARY KEY,
                address TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                tcp BOOLEAN NOT NULL,
                udp BOOLEAN NOT NULL,
                http BOOLEAN NOT NULL,
                socks5 BOOLEAN NOT NULL,
                datacenter BOOLEAN NOT NULL,
                residential BOOLEAN NOT NULL,
                mobile BOOLEAN NOT NULL,
                pool TEXT,
                country TEXT NOT NULL
            )",
        )
        .unwrap();

        // insert the rows
        for line in RAW_PROXIES_CSV.lines().take(n) {
            let proxy = Proxy::from(line.to_string());
            let statement = format!(
                "INSERT INTO proxies (id, address, username, password, tcp, udp, http, socks5, datacenter, residential, mobile, pool, country)
                VALUES ({}, '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, '{}')",
                proxy.id,
                proxy.address,
                proxy.username,
                proxy.password,
                proxy.tcp as i32,
                proxy.udp as i32,
                proxy.http as i32,
                proxy.socks5 as i32,
                proxy.datacenter as i32,
                proxy.residential as i32,
                proxy.mobile as i32,
                proxy.pool.map_or("NULL".to_owned(), |s| format!("'{}'", s.deref())),
                proxy.country.0,
            );
            conn.execute(&statement).unwrap();
        }

        SqlLiteProxyDB { conn }
    }

    fn get(&self, id: u64) -> Option<Cow<Proxy>> {
        let statement = format!("SELECT * FROM proxies WHERE id = {} LIMIT 1", id);
        let row = self
            .conn
            .prepare(&statement)
            .unwrap()
            .into_iter()
            .next()?
            .ok()?;
        let proxy = proxy_from_sql_row(row);
        Some(Cow::Owned(proxy))
    }

    fn any_tcp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let statement = format!(
            "SELECT * FROM proxies WHERE tcp = 1 AND pool = '{}' AND country = '{}' ORDER BY RANDOM() LIMIT 1",
            NormalizedString::from(pool).0, NormalizedString::from(country).0
        );
        let row = self
            .conn
            .prepare(&statement)
            .unwrap()
            .into_iter()
            .next()?
            .ok()?;
        let proxy = proxy_from_sql_row(row);
        Some(Cow::Owned(proxy))
    }

    fn any_socks5_isp(&self, pool: &str, country: &str) -> Option<Cow<Proxy>> {
        let statement = format!(
            "SELECT * FROM proxies WHERE socks5 = 1 AND datacenter = 1 AND residential = 1 AND pool = '{}' AND country = '{}' ORDER BY RANDOM() LIMIT 1",
            NormalizedString::from(pool).0, NormalizedString::from(country).0
        );
        let row = self
            .conn
            .prepare(&statement)
            .unwrap()
            .into_iter()
            .next()?
            .ok()?;
        let proxy = proxy_from_sql_row(row);
        Some(Cow::Owned(proxy))
    }
}

fn proxy_from_sql_row(row: Row) -> Proxy {
    Proxy {
        id: row.read::<i64, _>("id") as u64,
        address: row.read::<&str, _>("address").to_owned(),
        username: row.read::<&str, _>("username").to_owned(),
        password: row.read::<&str, _>("password").to_owned(),
        tcp: row.read::<i64, _>("tcp") != 0,
        udp: row.read::<i64, _>("udp") != 0,
        http: row.read::<i64, _>("http") != 0,
        socks5: row.read::<i64, _>("socks5") != 0,
        datacenter: row.read::<i64, _>("datacenter") != 0,
        residential: row.read::<i64, _>("residential") != 0,
        mobile: row.read::<i64, _>("mobile") != 0,
        pool: row.try_read::<&str, _>("pool").ok().map(Into::into),
        country: row.read::<&str, _>("country").into(),
    }
}

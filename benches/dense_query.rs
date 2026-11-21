//! Generated query builder through result access. Index construction and verification are untimed.
use divan::Bencher;
use venndb::VennDB;

#[derive(Debug, VennDB)]
struct Row {
    index: usize,
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    e: bool,
    f: bool,
    g: bool,
    h: bool,
    #[venndb(filter)]
    label: u8,
}

fn database(rows: usize, clustered: bool) -> RowDB {
    let mut state = 9357u64;
    RowDB::from_rows(
        (0..rows)
            .map(|index| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let mut flags = state as u8;
                if clustered && (index / 65536) % 8 != 0 {
                    flags = (flags & !3) | (1 << (index % 2));
                }
                let [a, b, c, d, e, f, g, h] = std::array::from_fn(|i| flags & (1 << i) != 0);
                Row {
                    index,
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    g,
                    h,
                    label: flags >> 4,
                }
            })
            .collect(),
    )
}

const SIZES: &[usize] = &[1000, 1_048_579, 8_388_611];

fn dense(bencher: Bencher, rows: usize, filters: usize, clustered: bool) {
    let db = database(rows, clustered);
    let run = || {
        let mut q = db.query();
        q.a(true).b(true);
        if filters >= 4 {
            q.c(true).d(true);
        }
        if filters == 8 {
            q.e(true).f(true).g(true).h(true);
        }
        q.execute()
    };
    let expected = db.iter().filter(|r| {
        r.a && r.b && (filters < 4 || r.c && r.d) && (filters < 8 || r.e && r.f && r.g && r.h)
    });
    assert!(
        run()
            .iter()
            .flat_map(|m| m.iter())
            .map(|r| r.index)
            .eq(expected.map(|r| r.index))
    );
    bencher.bench(|| run().map(|m| m.first().index));
}

#[divan::bench(args = SIZES, consts = [2, 4, 8])]
fn independent<const N: usize>(b: Bencher, rows: usize) {
    dense(b, rows, N, false);
}

#[divan::bench(args = SIZES, consts = [2, 4, 8])]
fn clustered<const N: usize>(b: Bencher, rows: usize) {
    dense(b, rows, N, true);
}

#[divan::bench(args = SIZES)]
fn single_label(b: Bencher, rows: usize) {
    let db = database(rows, false);
    let run = || db.query().label(3).execute();
    assert!(
        run()
            .iter()
            .flat_map(|m| m.iter())
            .map(|r| r.index)
            .eq(db.iter().filter(|r| r.label == 3).map(|r| r.index))
    );
    b.bench(|| run().map(|m| m.first().index));
}

fn main() {
    divan::main();
}

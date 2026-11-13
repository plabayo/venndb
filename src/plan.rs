//! Combining row sets into the rows that match a query.

use crate::rowset::{Ones, Repr, RowSet, WORD, select};
use crate::runtime::RuntimeColumn;

/// Start from the smallest set and test its rows one by one
/// as long as it holds less than 1 in `DRIVE_RATIO` rows,
/// combine whole bitmasks otherwise.
const DRIVE_RATIO: usize = 256;

/// The number of random rows tried before filtering all of them.
const RANDOM_TRIES: usize = 8;

/// The number of random rows of the database tried for a query,
/// before working out all rows that meet it.
const GUESSES: usize = 16;

/// Random rows are only tried if at least 1 row in `GUESS_RATIO` is expected to meet the query.
const GUESS_RATIO: f64 = 8.0;

/// A few row sets, of which the first ones are kept without allocating.
#[derive(Debug, Clone, Default)]
pub struct RowSets<'a> {
    first: [Option<&'a RowSet>; 3],
    more: Vec<&'a RowSet>,
}

impl<'a> RowSets<'a> {
    /// Create an empty list of sets.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            first: [None; 3],
            more: Vec::new(),
        }
    }

    /// Add a set, unless it is in the list already.
    pub fn push(&mut self, set: &'a RowSet) {
        if self.iter().any(|known| std::ptr::eq(known, set)) {
            return;
        }
        match self.first.iter_mut().find(|slot| slot.is_none()) {
            Some(slot) => *slot = Some(set),
            None => self.more.push(set),
        }
    }

    /// A list of the given sets, which the caller knows to be all different.
    pub(crate) fn distinct(sets: impl IntoIterator<Item = &'a RowSet>) -> Self {
        let mut sets = sets.into_iter();
        let mut first = [None; 3];
        first.iter_mut().for_each(|slot| *slot = sets.next());
        Self {
            first,
            more: sets.collect(),
        }
    }

    /// Iterate over the sets, in the order they were added.
    pub fn iter(&self) -> impl Iterator<Item = &'a RowSet> + Clone + '_ {
        self.first.iter().flatten().chain(&self.more).copied()
    }
}

impl<'a> FromIterator<&'a RowSet> for RowSets<'a> {
    fn from_iter<I: IntoIterator<Item = &'a RowSet>>(sets: I) -> Self {
        let mut this = Self::new();
        sets.into_iter().for_each(|set| this.push(set));
        this
    }
}

/// A single condition a row has to meet.
#[derive(Debug, Clone)]
pub enum Filter<'a> {
    /// The row is in the set.
    Is(&'a RowSet),
    /// The row is not in the set.
    Not(&'a RowSet),
    /// The row is in at least one of the sets.
    AnyOf(RowSets<'a>),
    /// The row is set, or not set, in the runtime column.
    Runtime(&'a RuntimeColumn, bool),
}

impl Filter<'_> {
    fn contains(&self, row: usize) -> bool {
        match self {
            Self::Is(set) => set.contains(row),
            Self::Not(set) => !set.contains(row),
            Self::AnyOf(sets) => sets.iter().any(|set| set.contains(row)),
            Self::Runtime(column, value) => column.get(row) == *value,
        }
    }

    /// The share of the rows that meet the filter at most, as far as it is known.
    fn share(&self, rows: usize) -> f64 {
        let rows_in = match self {
            Self::Is(set) => set.len(),
            Self::Not(set) => rows.saturating_sub(set.len()),
            Self::AnyOf(sets) => sets.iter().map(RowSet::len).sum(),
            Self::Runtime(..) => rows,
        };
        rows_in.min(rows) as f64 / rows as f64
    }

    /// The work it takes to walk over all rows of the filter, if that is possible.
    fn walk_cost(&self) -> Option<usize> {
        let cost = |set: &RowSet| match set.repr() {
            Repr::Sparse(rows) => rows.len(),
            Repr::Dense(words, ones) => words.len() + ones,
        };
        match self {
            Self::Is(set) => Some(cost(set)),
            Self::AnyOf(sets) => Some(sets.iter().map(cost).sum()),
            Self::Not(_) | Self::Runtime(..) => None,
        }
    }
}

/// The filters of a query: a row matches if it meets all of them.
///
/// A plan has a fixed number of slots, each holding at most one filter,
/// for the filters that can be set more than once. Other filters are added
/// with [`Plan::and`].
#[derive(Debug, Clone)]
pub struct Plan<'a> {
    rows: usize,
    slots: Vec<Option<Filter<'a>>>,
    extra: Vec<Filter<'a>>,
}

impl<'a> Plan<'a> {
    /// Create a plan for a database with the given number of rows.
    ///
    /// # Panics
    ///
    /// Panics if there are more rows than [`RowSet::MAX_ROWS`].
    #[must_use]
    pub fn new(rows: usize, slots: usize) -> Self {
        assert!(
            rows <= RowSet::MAX_ROWS,
            "venndb: a database holds at most u32::MAX rows"
        );
        Self {
            rows,
            slots: vec![None; slots],
            extra: Vec::new(),
        }
    }

    /// Remove all filters.
    pub fn reset(&mut self) {
        self.slots.fill(None);
        self.extra.clear();
    }

    /// Set the filter of a slot, replacing the one it had.
    ///
    /// # Panics
    ///
    /// Panics if the plan has no such slot.
    pub fn set(&mut self, slot: usize, filter: Filter<'a>) {
        self.slots[slot] = Some(filter);
    }

    /// Add a set to the [`Filter::AnyOf`] of a slot.
    ///
    /// Adding `None` adds no rows, but the slot does become a filter.
    /// A filter of another kind in the slot is replaced.
    ///
    /// # Panics
    ///
    /// Panics if the plan has no such slot.
    pub fn or(&mut self, slot: usize, set: Option<&'a RowSet>) {
        if !matches!(self.slots[slot], Some(Filter::AnyOf(_))) {
            self.slots[slot] = Some(Filter::AnyOf(RowSets::new()));
        }
        if let (Some(Filter::AnyOf(sets)), Some(set)) = (&mut self.slots[slot], set) {
            sets.push(set);
        }
    }

    /// Add a filter next to the ones in the slots.
    pub fn and(&mut self, filter: Filter<'a>) {
        self.extra.push(filter);
    }

    fn filters(&self) -> impl Iterator<Item = &Filter<'a>> + Clone {
        self.slots.iter().flatten().chain(&self.extra)
    }

    /// The rows that meet all filters, `None` if there are no such rows.
    #[must_use]
    pub fn execute(&self) -> Option<Matches<'a>> {
        if self.rows == 0 {
            return None;
        }
        let mut driver: Option<(usize, usize)> = None;
        let mut only = None;
        let mut count = 0;
        for (at, filter) in self.filters().enumerate() {
            count += 1;
            only = Some(filter);
            if let Some(cost) = filter.walk_cost() {
                if cost == 0 {
                    return None;
                }
                if driver.is_none_or(|(best, _)| cost < best) {
                    driver = Some((cost, at));
                }
            }
        }
        match (count, only, driver) {
            (0, ..) => Some(Matches(Repr2::All(self.rows))),
            (1, Some(Filter::Is(set)), _) if set.span() <= self.rows => {
                Some(Matches(Repr2::Set(set)))
            }
            (_, _, Some((cost, at))) if cost.saturating_mul(DRIVE_RATIO) <= self.rows => {
                self.walk(at)
            }
            _ => self.combine(),
        }
    }

    /// A random row that meets all filters, `None` if there are no such rows.
    ///
    /// Each of those rows is as likely as any other. A query that many rows meet
    /// is answered by trying random rows, without working out all rows that meet it.
    #[must_use]
    pub fn random(&self) -> Option<usize> {
        match self.guess(|_| true) {
            Guess::Found(row) => Some(row),
            Guess::Nothing => None,
            Guess::Unknown => self.execute().map(|matches| matches.random()),
        }
    }

    /// A random row that meets all filters and for which the predicate returns `true`.
    #[must_use]
    pub fn random_where(&self, mut predicate: impl FnMut(usize) -> bool) -> Option<usize> {
        match self.guess(&mut predicate) {
            Guess::Found(row) => Some(row),
            Guess::Nothing => None,
            Guess::Unknown => self.execute()?.random_where(predicate),
        }
    }

    /// Try random rows of the database, if enough of them are expected to meet the filters.
    fn guess(&self, mut accept: impl FnMut(usize) -> bool) -> Guess {
        if self.rows == 0 || self.filters().any(|filter| filter.walk_cost() == Some(0)) {
            return Guess::Nothing;
        }
        // as if the filters have nothing to do with each other, which is good enough to decide
        let share: f64 = self
            .filters()
            .map(|filter| filter.share(self.rows))
            .product();
        if share * GUESS_RATIO < 1.0 {
            return Guess::Unknown;
        }
        for _ in 0..GUESSES {
            let row = crate::rng::below(self.rows);
            if self.filters().all(|filter| filter.contains(row)) && accept(row) {
                return Guess::Found(row);
            }
        }
        Guess::Unknown
    }

    /// Walk over the rows of one filter, keeping those that meet the others.
    fn walk(&self, driver: usize) -> Option<Matches<'a>> {
        let keep = |row: &usize| {
            *row < self.rows
                && self
                    .filters()
                    .enumerate()
                    .all(|(at, filter)| at == driver || filter.contains(*row))
        };
        let mut found: Vec<u32> = Vec::new();
        match self.filters().nth(driver) {
            Some(Filter::Is(set)) => found.extend(set.iter().filter(keep).map(|row| row as u32)),
            Some(Filter::AnyOf(sets)) => {
                for set in sets.iter() {
                    found.extend(set.iter().filter(keep).map(|row| row as u32));
                }
                if sets.iter().nth(1).is_some() {
                    found.sort_unstable();
                    found.dedup();
                }
            }
            _ => unreachable!("not a filter to walk over"),
        }
        (!found.is_empty()).then_some(Matches(Repr2::Rows(found)))
    }

    /// Combine the filters as bitmasks of all rows.
    fn combine(&self) -> Option<Matches<'a>> {
        let len = self.rows.div_ceil(WORD);
        let start = self
            .filters()
            .enumerate()
            .find_map(|(at, filter)| match filter {
                Filter::Is(set) => match set.repr() {
                    Repr::Dense(words, _) => Some((at, words)),
                    Repr::Sparse(_) => None,
                },
                _ => None,
            });
        let mut mask = match start {
            Some((_, words)) => {
                let mut mask: Vec<u64> = Vec::with_capacity(len);
                mask.extend(words.iter().take(len));
                mask.resize(len, 0);
                mask
            }
            None => vec![u64::MAX; len],
        };
        let mut scratch = Vec::new();
        for (at, filter) in self.filters().enumerate() {
            if start.is_some_and(|(start, _)| start == at) {
                continue;
            }
            match filter {
                Filter::Is(set) => match set.repr() {
                    Repr::Dense(words, _) => and(&mut mask, words.iter().copied()),
                    Repr::Sparse(found) => {
                        scratch.clear();
                        scratch.resize(len, 0);
                        for row in found.iter().map(|row| *row as usize) {
                            if let Some(word) = mask.get(row / WORD) {
                                scratch[row / WORD] |= word & (1 << (row % WORD));
                            }
                        }
                        std::mem::swap(&mut mask, &mut scratch);
                    }
                },
                Filter::Not(set) => match set.repr() {
                    Repr::Dense(words, _) => {
                        for (word, other) in mask.iter_mut().zip(words) {
                            *word &= !other;
                        }
                    }
                    Repr::Sparse(found) => {
                        for row in found.iter().map(|row| *row as usize) {
                            if let Some(word) = mask.get_mut(row / WORD) {
                                *word &= !(1 << (row % WORD));
                            }
                        }
                    }
                },
                Filter::AnyOf(sets) => {
                    scratch.clear();
                    scratch.resize(len, 0);
                    for set in sets.iter() {
                        match set.repr() {
                            Repr::Dense(words, _) => {
                                for (word, other) in scratch.iter_mut().zip(words) {
                                    *word |= other;
                                }
                            }
                            Repr::Sparse(found) => {
                                for row in found.iter().map(|row| *row as usize) {
                                    if let Some(word) = scratch.get_mut(row / WORD) {
                                        *word |= 1 << (row % WORD);
                                    }
                                }
                            }
                        }
                    }
                    and(&mut mask, scratch.iter().copied());
                }
                Filter::Runtime(column, value) => column.filter(&mut mask, *value),
            }
        }
        if !self.rows.is_multiple_of(WORD) {
            mask[len - 1] &= (1 << (self.rows % WORD)) - 1;
        }
        let ones: usize = mask.iter().map(|word| word.count_ones() as usize).sum();
        (ones > 0).then_some(Matches(Repr2::Mask(mask, ones)))
    }
}

/// What trying random rows came to.
#[derive(Debug, PartialEq, Eq)]
enum Guess {
    /// A row that meets the query.
    Found(usize),
    /// No row can meet the query.
    Nothing,
    /// None of the rows tried meets the query, or none was tried.
    Unknown,
}

/// Keep the bits that are also set in the other words, which may be fewer.
fn and(mask: &mut [u64], other: impl ExactSizeIterator<Item = u64>) {
    let known = other.len().min(mask.len());
    for (word, other) in mask.iter_mut().zip(other) {
        *word &= other;
    }
    mask[known..].fill(0);
}

/// The rows that match a query, there is always at least one.
#[derive(Debug, Clone)]
pub struct Matches<'a>(Repr2<'a>);

#[derive(Debug, Clone)]
enum Repr2<'a> {
    /// All rows of a database with this many rows.
    All(usize),
    /// The rows of a set of the database itself.
    Set(&'a RowSet),
    /// The rows, sorted and unique.
    Rows(Vec<u32>),
    /// One bit per row, and the number of ones.
    Mask(Vec<u64>, usize),
}

impl Matches<'_> {
    /// The number of rows, never zero.
    #[expect(clippy::len_without_is_empty, reason = "matches are never empty")]
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.0 {
            Repr2::All(rows) => *rows,
            Repr2::Set(set) => set.len(),
            Repr2::Rows(rows) => rows.len(),
            Repr2::Mask(_, ones) => *ones,
        }
    }

    /// The `n`th row in increasing order, counting from zero.
    #[must_use]
    pub fn nth(&self, n: usize) -> Option<usize> {
        match &self.0 {
            Repr2::All(rows) => (n < *rows).then_some(n),
            Repr2::Set(set) => set.nth(n),
            Repr2::Rows(rows) => rows.get(n).map(|row| *row as usize),
            Repr2::Mask(words, _) => select(words, n),
        }
    }

    /// The first row.
    #[must_use]
    pub fn first(&self) -> usize {
        self.nth(0).expect("matches are never empty")
    }

    /// A random row.
    #[must_use]
    pub fn random(&self) -> usize {
        self.nth(crate::rng::below(self.len()))
            .expect("matches are never empty")
    }

    /// A random row for which the predicate returns `true`.
    ///
    /// Tries a few random rows first, to only test all rows if none of those do.
    #[must_use]
    pub fn random_where(&self, mut predicate: impl FnMut(usize) -> bool) -> Option<usize> {
        for _ in 0..RANDOM_TRIES.min(self.len()) {
            let row = self.random();
            if predicate(row) {
                return Some(row);
            }
        }
        self.filter(predicate).map(|matches| matches.random())
    }

    /// The rows for which the predicate returns `true`, `None` if there are none.
    #[must_use]
    pub fn filter(&self, mut predicate: impl FnMut(usize) -> bool) -> Option<Matches<'static>> {
        let rows: Vec<u32> = self
            .iter()
            .filter(|row| predicate(*row))
            .map(|row| row as u32)
            .collect();
        (!rows.is_empty()).then_some(Matches(Repr2::Rows(rows)))
    }

    /// Iterate over the rows, in increasing order.
    #[must_use]
    pub fn iter(&self) -> MatchesIter<'_> {
        MatchesIter(match &self.0 {
            Repr2::All(rows) => IterRepr::All(0..*rows),
            Repr2::Set(set) => IterRepr::Set(set.iter()),
            Repr2::Rows(rows) => IterRepr::Rows(rows.iter()),
            Repr2::Mask(words, _) => IterRepr::Mask(Ones::new(words)),
        })
    }
}

impl<'a> IntoIterator for &'a Matches<'_> {
    type Item = usize;
    type IntoIter = MatchesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the rows of [`Matches`].
#[derive(Debug, Clone)]
pub struct MatchesIter<'a>(IterRepr<'a>);

#[derive(Debug, Clone)]
enum IterRepr<'a> {
    All(std::ops::Range<usize>),
    Set(crate::rowset::Iter<'a>),
    Rows(std::slice::Iter<'a, u32>),
    Mask(Ones<'a>),
}

impl Iterator for MatchesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match &mut self.0 {
            IterRepr::All(rows) => rows.next(),
            IterRepr::Set(rows) => rows.next(),
            IterRepr::Rows(rows) => rows.next().map(|row| *row as usize),
            IterRepr::Mask(ones) => ones.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Random(u64);

    impl Random {
        fn next(&mut self, limit: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % limit as u64) as usize
        }

        /// A set with about one in `one_in` of the rows.
        fn set(&mut self, rows: usize, one_in: usize) -> RowSet {
            (0..rows).filter(|_| self.next(one_in) == 0).collect()
        }
    }

    fn naive(rows: usize, filters: &[Filter<'_>]) -> Vec<usize> {
        (0..rows)
            .filter(|row| filters.iter().all(|filter| filter.contains(*row)))
            .collect()
    }

    fn found(matches: Option<Matches<'_>>) -> Vec<usize> {
        let Some(matches) = matches else {
            return Vec::new();
        };
        let rows: Vec<usize> = matches.iter().collect();
        assert!(!rows.is_empty(), "matches are never empty");
        assert_eq!(matches.len(), rows.len());
        assert_eq!(matches.first(), rows[0]);
        for (n, row) in rows.iter().enumerate() {
            assert_eq!(matches.nth(n), Some(*row));
        }
        assert_eq!(matches.nth(rows.len()), None);
        assert!(rows.contains(&matches.random()));
        assert!((&matches).into_iter().eq(rows.iter().copied()));
        rows
    }

    #[test]
    fn both_ways_agree_with_a_scan() {
        let mut random = Random(0x1234_5678_9ABC_DEF1);
        for rows in [1, 63, 64, 65, 200, 1000, 5000] {
            let column = RuntimeColumn::new();
            // also rows the database does not have
            for row in 0..rows + 70 {
                column.set(row, random.next(3) == 0);
            }
            let sets: Vec<RowSet> = [1, 2, 3, 10, 50, 100, 400, 1, 7, 300]
                .into_iter()
                .map(|one_in| {
                    let span = rows + random.next(2) * 70;
                    random.set(span, one_in)
                })
                .collect();
            for _ in 0..300 {
                let filters: Vec<Filter<'_>> = (0..1 + random.next(4))
                    .map(|_| {
                        let set = &sets[random.next(sets.len())];
                        match random.next(6) {
                            0 | 1 => Filter::Is(set),
                            2 => Filter::Not(set),
                            3 | 4 => Filter::AnyOf(
                                (0..random.next(4))
                                    .map(|_| &sets[random.next(sets.len())])
                                    .collect(),
                            ),
                            _ => Filter::Runtime(&column, random.next(2) == 0),
                        }
                    })
                    .collect();
                let expected = naive(rows, &filters);
                let mut plan = Plan::new(rows, 0);
                filters.iter().for_each(|filter| plan.and(filter.clone()));
                assert_eq!(found(plan.combine()), expected, "{filters:?}");
                for (at, filter) in filters.iter().enumerate() {
                    if filter.walk_cost().is_some() {
                        assert_eq!(found(plan.walk(at)), expected, "{filters:?}");
                    }
                }
                assert_eq!(found(plan.execute()), expected, "{filters:?}");
                // a random row is one of them, however it is found
                for _ in 0..3 {
                    let row = plan.random();
                    assert_eq!(row.is_some(), !expected.is_empty(), "{filters:?}");
                    assert!(row.is_none_or(|row| expected.contains(&row)), "{filters:?}");
                    let odd = plan.random_where(|row| row % 2 == 1);
                    assert_eq!(odd.is_some(), expected.iter().any(|row| row % 2 == 1));
                    assert!(odd.is_none_or(|row| row % 2 == 1 && expected.contains(&row)));
                }
                match plan.guess(|_| true) {
                    Guess::Found(row) => assert!(expected.contains(&row), "{filters:?}"),
                    Guess::Nothing => assert!(expected.is_empty(), "{filters:?}"),
                    Guess::Unknown => (),
                }
            }
        }
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    #[should_panic(expected = "at most u32::MAX rows")]
    fn rows_have_to_fit() {
        let _ = Plan::new(RowSet::MAX_ROWS + 1, 0);
    }

    #[test]
    fn random_reaches_every_row() {
        let odd: RowSet = (0..640).filter(|row| row % 2 == 1).collect();
        let few: RowSet = [3, 500, 77].into_iter().collect();
        let column = RuntimeColumn::new();
        let mut plans = [
            Plan::new(8, 0),
            Plan::new(640, 0),
            Plan::new(640, 0),
            Plan::new(640, 0),
        ];
        plans[1].and(Filter::Not(&odd));
        plans[1].and(Filter::Runtime(&column, false));
        plans[2].and(Filter::Is(&few));
        plans[3].and(Filter::Is(&odd));
        plans[3].and(Filter::Not(&few));
        for plan in &plans {
            let matches = plan.execute().unwrap();
            let rows: Vec<usize> = matches.iter().collect();
            let mut seen = std::collections::BTreeSet::new();
            for _ in 0..rows.len() * 40 {
                seen.insert(matches.random());
                let row = matches.random_where(|row| row % 3 != 0).unwrap();
                assert!(row % 3 != 0 && rows.contains(&row));
            }
            assert!(seen.iter().eq(rows.iter()), "{seen:?}");
        }
    }

    #[test]
    fn guesses_when_many_rows_match() {
        let rows = 6400;
        let half: RowSet = (0..rows).filter(|row| row % 2 == 0).collect();
        let few: RowSet = (0..rows).filter(|row| row % 100 == 0).collect();
        let none = RowSet::new();
        let column = RuntimeColumn::new();

        // nothing to filter on, or filters most rows meet: a random row will do
        let mut plan = Plan::new(rows, 0);
        assert!(matches!(plan.guess(|_| true), Guess::Found(_)));
        plan.and(Filter::Is(&half));
        plan.and(Filter::Not(&few));
        plan.and(Filter::Runtime(&column, false));
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..2000 {
            let Guess::Found(row) = plan.guess(|_| true) else {
                panic!("1 row in 2 meets the filters");
            };
            assert!(row % 2 == 0 && row % 100 != 0);
            seen.insert(row);
            seen.insert(plan.random().unwrap());
        }
        // spread over all of them
        assert!(seen.len() > 1000 && seen.first() < Some(&100) && seen.last() > Some(&6300));
        // a row the caller refuses is not returned
        assert_eq!(plan.guess(|_| false), Guess::Unknown);
        assert_eq!(plan.random_where(|_| false), None);
        assert_eq!(plan.random_where(|row| row == 42), Some(42));

        // too few rows meet the filters to hit one by chance
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&few));
        assert_eq!(plan.guess(|_| true), Guess::Unknown);
        assert!(plan.random().unwrap().is_multiple_of(100));
        // exactly 1 row in 8 is still tried, less is not
        let eighth: RowSet = (0..rows).filter(|row| row % 8 == 0).collect();
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&eighth));
        assert!((0..50).any(|_| matches!(plan.guess(|_| true), Guess::Found(_))));
        plan.and(Filter::Is(&half));
        assert_eq!(plan.guess(|_| true), Guess::Unknown);

        // no row at all
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&none));
        assert_eq!(plan.guess(|_| true), Guess::Nothing);
        assert_eq!((plan.random(), plan.random_where(|_| true)), (None, None));
        assert_eq!(Plan::new(0, 0).guess(|_| true), Guess::Nothing);
        // rows beyond those of the database are never guessed
        let beyond: RowSet = (0..rows * 2).collect();
        let mut plan = Plan::new(10, 0);
        plan.and(Filter::Is(&beyond));
        assert!((0..200).all(|_| plan.random().unwrap() < 10));
    }

    #[test]
    fn many_distinct_sets() {
        let sets: Vec<RowSet> = (0..10).map(|row| [row].into_iter().collect()).collect();
        let list = RowSets::distinct(&sets);
        assert!(list.iter().eq(sets.iter()));
        assert_eq!(RowSets::distinct(&sets[..2]).iter().count(), 2);
        let mut list: RowSets<'_> = sets.iter().chain(&sets).collect();
        assert!(list.iter().eq(sets.iter()));
        list.push(&sets[9]);
        assert_eq!(list.iter().count(), 10);
    }

    #[test]
    fn no_filters_match_all_rows() {
        assert!(Plan::new(0, 0).execute().is_none());
        assert_eq!(found(Plan::new(3, 2).execute()), [0, 1, 2]);
    }

    #[test]
    fn a_single_set_is_borrowed() {
        let set: RowSet = [1, 5].into_iter().collect();
        let mut plan = Plan::new(6, 0);
        plan.and(Filter::Is(&set));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Set(_)));
        assert_eq!(found(Some(matches)), [1, 5]);
        // a set with rows the database does not have is not used as is
        let mut plan = Plan::new(5, 0);
        plan.and(Filter::Is(&set));
        assert_eq!(found(plan.execute()), [1]);
    }

    #[test]
    fn slots() {
        let (a, b, empty): (RowSet, RowSet, RowSet) = (
            [0, 1].into_iter().collect(),
            [1, 2].into_iter().collect(),
            RowSet::new(),
        );
        let mut plan = Plan::new(4, 2);
        plan.set(0, Filter::Is(&a));
        plan.set(0, Filter::Not(&a));
        assert_eq!(found(plan.execute()), [2, 3]);
        plan.or(0, Some(&a));
        plan.or(0, Some(&a));
        assert!(matches!(&plan.slots[0], Some(Filter::AnyOf(sets)) if sets.iter().count() == 1));
        plan.or(0, Some(&b));
        plan.or(0, None);
        assert_eq!(found(plan.execute()), [0, 1, 2]);
        plan.or(1, Some(&b));
        assert_eq!(found(plan.execute()), [1, 2]);
        plan.and(Filter::Not(&a));
        assert_eq!(found(plan.execute()), [2]);
        plan.reset();
        assert_eq!(found(plan.execute()), [0, 1, 2, 3]);
        plan.or(1, None);
        assert_eq!(found(plan.execute()), []);
        plan.reset();
        plan.and(Filter::Is(&empty));
        plan.and(Filter::Not(&a));
        assert_eq!(found(plan.execute()), []);
    }

    #[test]
    fn small_sets_are_walked() {
        let rows = DRIVE_RATIO * 40;
        let small: RowSet = (0..40).map(|row| row * 200).collect();
        let big: RowSet = (0..rows).filter(|row| row % 2 == 0).collect();
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&big));
        plan.and(Filter::Is(&small));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Rows(_)));
        assert!(matches.iter().eq((0..40).map(|row| row * 200)));
        // one row too many to walk
        let small: RowSet = (0..41).map(|row| row * 200).collect();
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&big));
        plan.and(Filter::Is(&small));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Mask(..)));
        assert!(matches.iter().eq((0..41).map(|row| row * 200)));
    }

    #[test]
    fn filter_and_random_where() {
        let mut plan = Plan::new(100, 0);
        let odd: RowSet = (0..100).filter(|row| row % 2 == 1).collect();
        plan.and(Filter::Not(&odd));
        let matches = plan.execute().unwrap();
        assert_eq!(matches.len(), 50);
        let filtered = matches.filter(|row| row >= 90).unwrap();
        assert_eq!(found(Some(filtered)), [90, 92, 94, 96, 98]);
        assert!(matches.filter(|row| row > 98).is_none());

        for _ in 0..50 {
            assert_eq!(matches.random_where(|row| row == 42), Some(42));
            assert!(matches.random_where(|row| row < 10).unwrap() < 10);
        }
        assert_eq!(matches.random_where(|_| false), None);
        // a single row is not tested more often than needed
        let one = Plan::new(1, 0).execute().unwrap();
        let mut calls = 0;
        assert_eq!(
            one.random_where(|_| {
                calls += 1;
                false
            }),
            None
        );
        assert_eq!(calls, 2);
    }
}

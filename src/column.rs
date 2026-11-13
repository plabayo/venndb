//! The columns a database is made of.

use crate::plan::{Filter, Plan};
use crate::rowset::RowSet;
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::Hash;

/// The wildcard rows are part of every query on their column,
/// which makes them worth a bitmask long before other rows are.
const ANY_DENSE_RATIO: usize = 1024;

/// The hash map of the standard library, with a faster hash function.
type HashMap<K, V> = std::collections::HashMap<K, V, foldhash::fast::RandomState>;

/// The rows of a unique key.
#[derive(Debug, Clone)]
pub struct KeyIndex<K> {
    rows: HashMap<K, u32>,
}

impl<K> Default for KeyIndex<K> {
    fn default() -> Self {
        Self {
            rows: HashMap::default(),
        }
    }
}

impl<K: Hash + Eq> KeyIndex<K> {
    /// Create an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty index with room for the given number of keys.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: HashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// The row of a key.
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.rows.get(key).map(|row| *row as usize)
    }

    /// Return `true` if the key is known.
    #[must_use]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.rows.contains_key(key)
    }

    /// Add a key, returns `false` and changes nothing if the key was known already.
    ///
    /// # Panics
    ///
    /// Panics if the row is not below [`RowSet::MAX_ROWS`].
    pub fn insert(&mut self, key: K, row: usize) -> bool {
        let row = crate::rowset::stored(row);
        match self.rows.entry(key) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(row);
                true
            }
        }
    }
}

/// A column of `bool` values.
#[derive(Debug, Clone, Default)]
pub struct BoolColumn {
    yes: RowSet,
    /// Only kept once a row without value is added,
    /// until then `false` is all that is not `true`.
    no: Option<RowSet>,
    len: usize,
}

impl BoolColumn {
    /// Create an empty column.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            yes: RowSet::new(),
            no: None,
            len: 0,
        }
    }

    /// Add the value of the next row,
    /// a row without value matches neither `true` nor `false`.
    ///
    /// The column has to get a value for every row of the database:
    /// what a filter on it does for rows it knows nothing of is not defined.
    pub fn push(&mut self, value: Option<bool>) {
        let row = self.len;
        self.len += 1;
        match value {
            Some(true) => {
                self.yes.insert(row);
            }
            Some(false) => {
                if let Some(no) = &mut self.no {
                    no.insert(row);
                }
            }
            None => {
                if self.no.is_none() {
                    let yes = &self.yes;
                    self.no = Some((0..row).filter(|row| !yes.contains(*row)).collect());
                }
            }
        }
    }

    /// Make the slot of the plan filter on the given value.
    pub fn filter<'a>(&'a self, plan: &mut Plan<'a>, slot: usize, value: bool) {
        plan.set(
            slot,
            match (value, &self.no) {
                (true, _) => Filter::Is(&self.yes),
                (false, Some(no)) => Filter::Is(no),
                (false, None) => Filter::Not(&self.yes),
            },
        );
    }

    /// Release the memory the column does not need.
    pub fn shrink_to_fit(&mut self) {
        self.yes.shrink_to_fit();
        if let Some(no) = &mut self.no {
            no.shrink_to_fit();
        }
    }
}

/// A column of values, with the rows of each distinct value.
///
/// A row can have one value, several, or none at all. It can also
/// be a wildcard, in which case it matches any value asked for.
#[derive(Debug, Clone)]
pub struct ValueColumn<T> {
    slots: HashMap<T, u32>,
    rows: Vec<RowSet>,
    any: RowSet,
}

impl<T> Default for ValueColumn<T> {
    fn default() -> Self {
        Self {
            slots: HashMap::default(),
            rows: Vec::new(),
            any: RowSet::new(),
        }
    }
}

impl<T: Hash + Eq> ValueColumn<T> {
    /// Create an empty column.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a value of a row.
    pub fn push(&mut self, row: usize, value: &T)
    where
        T: Clone,
    {
        let slot = if let Some(slot) = self.slots.get(value) {
            *slot as usize
        } else {
            let slot = self.rows.len();
            let stored = u32::try_from(slot).expect("venndb: too many distinct values");
            self.slots.insert(value.clone(), stored);
            self.rows.push(RowSet::new());
            slot
        };
        self.rows[slot].insert(row);
    }

    /// Make a row a wildcard, matching any value.
    pub fn push_any(&mut self, row: usize) {
        self.any.insert_at(row, ANY_DENSE_RATIO);
    }

    /// The rows with the given value, wildcards not included.
    #[must_use]
    pub fn rows<Q>(&self, value: &Q) -> Option<&RowSet>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.slots.get(value).map(|slot| &self.rows[*slot as usize])
    }

    /// The wildcard rows.
    #[must_use]
    pub const fn any(&self) -> &RowSet {
        &self.any
    }

    /// The distinct values of the column and their rows.
    pub fn values(&self) -> impl Iterator<Item = (&T, &RowSet)> {
        self.slots
            .iter()
            .map(|(value, slot)| (value, &self.rows[*slot as usize]))
    }

    /// Make the slot of the plan also match the given value,
    /// next to the values it matches already.
    ///
    /// Pass `any` as `true` to only match the wildcard rows.
    pub fn filter<'a, Q>(&'a self, plan: &mut Plan<'a>, slot: usize, value: &Q, any: bool)
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        plan.or(slot, self.rows(value).filter(|_| !any));
        plan.or(slot, Some(&self.any).filter(|any| !any.is_empty()));
    }

    /// Make the plan match the values for which the predicate returns `true`.
    ///
    /// The predicate is called once per distinct value, not once per row.
    pub fn filter_where<'a>(&'a self, plan: &mut Plan<'a>, mut predicate: impl FnMut(&T) -> bool) {
        // the sets of distinct values are distinct: no need to look for doubles
        let sets = self
            .values()
            .filter(|(value, rows)| !rows.is_empty() && predicate(value))
            .map(|(_, rows)| rows)
            .chain(Some(&self.any).filter(|any| !any.is_empty()));
        let sets = crate::plan::RowSets::distinct(sets);
        plan.and(Filter::AnyOf(sets));
    }

    /// Release the memory the column does not need.
    pub fn shrink_to_fit(&mut self) {
        self.slots.shrink_to_fit();
        self.rows.shrink_to_fit();
        self.rows.iter_mut().for_each(RowSet::shrink_to_fit);
        self.any.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(plan: &Plan<'_>) -> Vec<usize> {
        plan.execute()
            .map(|matches| matches.iter().collect())
            .unwrap_or_default()
    }

    #[test]
    fn key_index() {
        let mut index = KeyIndex::with_capacity(2);
        assert!(index.insert("a".to_owned(), 0));
        assert!(index.insert("b".to_owned(), 7));
        assert!(!index.insert("a".to_owned(), 9));
        assert_eq!(index.get("a"), Some(0));
        assert_eq!(index.get("b"), Some(7));
        assert_eq!(index.get("c"), None);
        assert!(index.contains("b") && !index.contains("c"));
        assert!(!KeyIndex::<u8>::new().contains(&1));
    }

    #[test]
    #[should_panic(expected = "at most u32::MAX rows")]
    fn key_row_has_to_fit() {
        KeyIndex::new().insert(1, RowSet::MAX_ROWS);
    }

    #[test]
    fn bool_column() {
        let mut column = BoolColumn::new();
        for value in [true, false, false, true] {
            column.push(Some(value));
        }
        column.shrink_to_fit();
        let mut plan = Plan::new(4, 1);
        column.filter(&mut plan, 0, true);
        assert_eq!(rows(&plan), [0, 3]);
        column.filter(&mut plan, 0, false);
        assert_eq!(rows(&plan), [1, 2]);
    }

    #[test]
    fn bool_column_without_value() {
        let mut column = BoolColumn::new();
        for value in [Some(true), Some(false)] {
            column.push(value);
        }
        let mut plan = Plan::new(2, 1);
        column.filter(&mut plan, 0, false);
        assert_eq!(rows(&plan), [1]);
        for value in [None, Some(false), None, Some(true)] {
            column.push(value);
        }
        column.shrink_to_fit();
        let mut plan = Plan::new(6, 1);
        column.filter(&mut plan, 0, true);
        assert_eq!(rows(&plan), [0, 5]);
        column.filter(&mut plan, 0, false);
        assert_eq!(rows(&plan), [1, 3]);
    }

    fn letters() -> ValueColumn<char> {
        let mut column = ValueColumn::new();
        column.push(0, &'a');
        column.push(1, &'b');
        column.push_any(2);
        column.push(3, &'a');
        column.push(3, &'c');
        column.push(3, &'c');
        column
    }

    #[test]
    fn value_column() {
        let mut column = letters();
        column.shrink_to_fit();
        assert_eq!(
            column.rows(&'a').unwrap().iter().collect::<Vec<_>>(),
            [0, 3]
        );
        assert_eq!(column.rows(&'c').unwrap().len(), 1);
        assert!(column.rows(&'z').is_none());
        assert_eq!(column.any().iter().collect::<Vec<_>>(), [2]);
        let mut values: Vec<_> = column.values().map(|(v, rows)| (*v, rows.len())).collect();
        values.sort_unstable();
        assert_eq!(values, [('a', 2), ('b', 1), ('c', 1)]);
    }

    #[test]
    fn wildcards_turn_dense_early() {
        let mut column = ValueColumn::new();
        // 1 row in 1000: far too few for a value, enough for the wildcards
        for row in 0..20 {
            column.push(row * 1000, &'a');
            column.push_any(row * 1000 + 1);
        }
        let mut plan = Plan::new(20_000, 1);
        column.filter(&mut plan, 0, &'a', true);
        let matches = plan.execute().unwrap();
        assert!(matches.iter().eq((0..20).map(|row| row * 1000 + 1)));
        let words = |set: &RowSet| matches!(set.repr(), crate::rowset::Repr::Dense(..));
        assert!(words(column.any()) && !words(column.rows(&'a').unwrap()));
        // but not at 1 row in 2000
        let mut column = ValueColumn::<char>::new();
        (0..20).for_each(|row| column.push_any(row * 2000));
        assert!(!words(column.any()));
    }

    #[test]
    fn value_filter() {
        let column = letters();
        let mut plan = Plan::new(4, 1);
        column.filter(&mut plan, 0, &'a', false);
        assert_eq!(rows(&plan), [0, 2, 3]);
        column.filter(&mut plan, 0, &'b', false);
        assert_eq!(rows(&plan), [0, 1, 2, 3]);
        plan.reset();
        column.filter(&mut plan, 0, &'z', false);
        assert_eq!(rows(&plan), [2]);
        plan.reset();
        column.filter(&mut plan, 0, &'a', true);
        assert_eq!(rows(&plan), [2]);
    }

    #[test]
    fn value_filter_without_wildcards() {
        let mut column = ValueColumn::new();
        column.push(0, &1);
        column.push(1, &2);
        let mut plan = Plan::new(2, 1);
        column.filter(&mut plan, 0, &3, false);
        assert_eq!(rows(&plan), []);
        column.filter(&mut plan, 0, &2, false);
        assert_eq!(rows(&plan), [1]);
        plan.reset();
        column.filter(&mut plan, 0, &1, true);
        assert_eq!(rows(&plan), []);
    }

    #[test]
    fn value_filter_where() {
        let column = letters();
        let mut plan = Plan::new(4, 0);
        column.filter_where(&mut plan, |letter| *letter > 'a');
        assert_eq!(rows(&plan), [1, 2, 3]);
        plan.reset();
        column.filter_where(&mut plan, |_| false);
        assert_eq!(rows(&plan), [2]);

        let mut column = ValueColumn::new();
        column.push(5, &1);
        let mut plan = Plan::new(6, 0);
        column.filter_where(&mut plan, |_| false);
        assert_eq!(rows(&plan), []);
        let mut calls = 0;
        plan.reset();
        column.filter_where(&mut plan, |_| {
            calls += 1;
            true
        });
        assert_eq!((rows(&plan), calls), (vec![5], 1));
    }
}

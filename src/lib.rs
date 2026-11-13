#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

#[cfg(feature = "derive")]
pub use venndb_macros::VennDB;

mod column;
mod plan;
mod rng;
mod rowset;
mod runtime;

pub use column::{BoolColumn, KeyIndex, ValueColumn};
pub use plan::{Filter, Matches, MatchesIter, Plan, RowSets};
pub use rowset::{Iter as RowSetIter, RowSet};
pub use runtime::RuntimeColumn;

/// A trait that types can implement in order to support `#[venndb(any)]` attribute filters.
pub trait Any {
    /// Returns true if the value is considered to be "any" within the context of the type.
    ///
    /// # Example
    ///
    /// ```
    /// use venndb::Any;
    ///
    /// #[derive(Debug)]
    /// struct MyString(String);
    ///
    /// impl Any for MyString {
    ///    fn is_any(&self) -> bool {
    ///       self.0 == "*"
    ///   }
    /// }
    ///
    /// let my_string = MyString("*".to_string());
    /// assert!(my_string.is_any());
    ///
    /// let my_string = MyString("hello".to_string());
    /// assert!(!my_string.is_any());
    /// ```
    fn is_any(&self) -> bool;
}

impl<T: Any> Any for &T {
    fn is_any(&self) -> bool {
        T::is_any(*self)
    }
}

impl<T: Any> Any for Option<T> {
    fn is_any(&self) -> bool {
        match self {
            Some(value) => value.is_any(),
            None => false,
        }
    }
}

impl<T: Any> Any for std::sync::Arc<T> {
    fn is_any(&self) -> bool {
        T::is_any(&**self)
    }
}

impl<T: Any> Any for std::rc::Rc<T> {
    fn is_any(&self) -> bool {
        T::is_any(&**self)
    }
}

impl<T: Any> Any for Box<T> {
    fn is_any(&self) -> bool {
        T::is_any(&**self)
    }
}

/// A trait that types can implement in order to support `#[venndb(filter, set)]`
/// attribute filters: a field with several values, of which a row matches each.
///
/// Use `#[venndb(filter, members = path)]` instead for a type that is not yours.
///
/// # Example
///
/// ```
/// use venndb::FilterSet;
///
/// struct Tags(Vec<String>);
///
/// impl FilterSet for Tags {
///     type Item = String;
///
///     fn for_each_member(&self, f: impl FnMut(&String)) {
///         self.0.iter().for_each(f);
///     }
/// }
/// ```
pub trait FilterSet {
    /// The type of the values in the set.
    type Item;

    /// Call the function once for each value in the set.
    fn for_each_member(&self, f: impl FnMut(&Self::Item));
}

impl<T> FilterSet for [T] {
    type Item = T;

    fn for_each_member(&self, f: impl FnMut(&T)) {
        self.iter().for_each(f);
    }
}

impl<T, const N: usize> FilterSet for [T; N] {
    type Item = T;

    fn for_each_member(&self, f: impl FnMut(&T)) {
        self.iter().for_each(f);
    }
}

impl<T> FilterSet for Vec<T> {
    type Item = T;

    fn for_each_member(&self, f: impl FnMut(&T)) {
        self.iter().for_each(f);
    }
}

impl<T, S> FilterSet for std::collections::HashSet<T, S> {
    type Item = T;

    fn for_each_member(&self, f: impl FnMut(&T)) {
        self.iter().for_each(f);
    }
}

impl<T> FilterSet for std::collections::BTreeSet<T> {
    type Item = T;

    fn for_each_member(&self, f: impl FnMut(&T)) {
        self.iter().for_each(f);
    }
}

impl<S: FilterSet + ?Sized> FilterSet for &S {
    type Item = S::Item;

    fn for_each_member(&self, f: impl FnMut(&S::Item)) {
        S::for_each_member(self, f);
    }
}

impl<S: FilterSet + ?Sized> FilterSet for Box<S> {
    type Item = S::Item;

    fn for_each_member(&self, f: impl FnMut(&S::Item)) {
        S::for_each_member(self, f);
    }
}

impl<S: FilterSet + ?Sized> FilterSet for std::sync::Arc<S> {
    type Item = S::Item;

    fn for_each_member(&self, f: impl FnMut(&S::Item)) {
        S::for_each_member(self, f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};
    use std::sync::Arc;

    struct Star(char);

    impl Any for Star {
        fn is_any(&self) -> bool {
            self.0 == '*'
        }
    }

    #[test]
    fn any_through_wrappers() {
        for (value, any) in [('*', true), ('a', false)] {
            assert_eq!(Star(value).is_any(), any);
            assert_eq!(<&Star as Any>::is_any(&&Star(value)), any);
            assert_eq!(Some(Star(value)).is_any(), any);
            assert_eq!(Arc::new(Star(value)).is_any(), any);
            assert_eq!(std::rc::Rc::new(Star(value)).is_any(), any);
            assert_eq!(Box::new(Star(value)).is_any(), any);
        }
        assert!(!None::<Star>.is_any());
    }

    fn members<S: FilterSet<Item = u8> + ?Sized>(set: &S) -> Vec<u8> {
        let mut members = Vec::new();
        set.for_each_member(|member| members.push(*member));
        members.sort_unstable();
        members
    }

    #[test]
    fn members_of_sets() {
        assert_eq!(members(&vec![3, 1]), [1, 3]);
        assert_eq!(members(&[3, 1][..]), [1, 3]);
        assert_eq!(members(&[3, 1]), [1, 3]);
        assert_eq!(members(&HashSet::from([3, 1])), [1, 3]);
        assert_eq!(members(&BTreeSet::from([3, 1])), [1, 3]);
        assert_eq!(members(&&vec![3, 1]), [1, 3]);
        assert_eq!(members(&Box::new(vec![3, 1])), [1, 3]);
        assert_eq!(members(&Arc::<[u8]>::from([3, 1])), [1, 3]);
        assert_eq!(members(&Vec::new()), [0u8; 0]);
    }
}

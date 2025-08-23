#![doc = include_str!("../README.md")]

pub use venndb_macros::VennDB;

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

#[doc(hidden)]
pub mod __internal {
    //! Hidden thirdparty dependencies for venndb,
    //! not to be relied upon directly, as they may change at any time.

    pub use bitvec::{order::Lsb0, slice::IterOnes, vec::BitVec};
    pub use hashbrown::HashMap;
    use rand::Rng;

    #[must_use]
    /// Generate a random `usize`.
    pub fn rand_range(limit: usize) -> usize {
        rand::rng().random_range(0..limit)
    }

    pub mod hash_map {
        //! Internal types related to hash map.

        pub use hashbrown::hash_map::Entry;
    }
}

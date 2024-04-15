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

#[doc(hidden)]
pub mod __internal {
    //! Hidden thirdparty dependencies for venndb,
    //! not to be relied upon directly, as they may change at any time.

    pub use bitvec::{order::Lsb0, slice::IterOnes, vec::BitVec};
    pub use hashbrown::HashMap;

    /// Generate a random `usize`.
    pub fn rand_usize() -> usize {
        use rand::Rng;

        rand::thread_rng().gen()
    }

    pub mod hash_map {
        //! Internal types related to hash map.

        pub use hashbrown::hash_map::Entry;
    }
}

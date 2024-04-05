pub use venndb_macros::VennDB;

#[doc(hidden)]
pub mod internal {
    //! Hidden thirdparty dependencies for venndb,
    //! not to be relied upon directly, as they may change at any time.

    pub use hashbrown::HashMap;
}

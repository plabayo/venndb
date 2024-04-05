pub use venndb_macros::VennDB;

#[doc(hidden)]
pub mod __internal {
    //! Hidden thirdparty dependencies for venndb,
    //! not to be relied upon directly, as they may change at any time.

    pub use bitvec::{bitvec, order::Lsb0, slice::IterOnes, vec::BitVec};
    pub use hashbrown::HashMap;
}

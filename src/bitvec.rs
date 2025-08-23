#![allow(dead_code)]
//! Fork from <https://docs.rs/vob/3.0.6/src/vob/lib.rs.html#138-145>
//! Original License: <https://github.com/softdevteam/vob/blob/master/LICENSE-MIT>

#[must_use]
#[derive(Debug, Clone, Default)]
pub struct BitVec {
    len: usize,
    data: Vec<usize>,
}

#[derive(Debug)]
pub struct IterOnes<'a> {
    index: usize,
    bv: &'a BitVec,
}

impl BitVec {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            len: 0,
            data: Vec::with_capacity(blocks_required(capacity)),
        }
    }

    pub fn repeat(value: bool, len: usize) -> Self {
        let mut v = Self {
            len,
            data: vec![if value { !0 } else { 0 }; blocks_required(len)],
        };
        v.mask_last_block();
        v
    }

    #[must_use]
    pub fn iter_ones(&self) -> IterOnes<'_> {
        IterOnes { index: 0, bv: self }
    }

    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.iter_ones().count()
    }

    #[must_use]
    pub fn any(&self) -> bool {
        self.iter_ones().next().is_some()
    }

    pub fn push(&mut self, value: bool) {
        debug_assert_eq!(self.data.len(), blocks_required(self.len));
        if self.len % BITS_PER_BLOCK == 0 {
            self.data.push(0);
        }
        let i = self.len;
        self.len = i.checked_add(1).expect("Overflow detected");
        self.set(i, value);
    }

    fn set(&mut self, index: usize, value: bool) -> bool {
        if index >= self.len {
            panic!(
                "Index out of bounds: the len is {} but the index is {}",
                self.len, index
            );
        }
        let msk = 1 << (index % BITS_PER_BLOCK);
        let off = block_offset(index);
        let old_v = self.data[off];
        let new_v = if value { old_v | msk } else { old_v & !msk };
        if new_v != old_v {
            self.data[off] = new_v;
            true
        } else {
            false
        }
    }

    pub fn or(&mut self, other: &Self) -> bool {
        let mut chngd = false;
        for (self_blk, other_blk) in self
            .data
            .iter_mut()
            .zip(other.data.iter().chain(std::iter::repeat(&0)))
        {
            let old_v = *self_blk;
            let new_v = old_v | *other_blk;
            *self_blk = new_v;
            chngd |= old_v != new_v;
        }
        // We don't need to mask the last block per our assumptions
        chngd
    }

    pub fn and(&mut self, other: &Self) -> bool {
        let mut chngd = false;
        for (self_blk, other_blk) in self
            .data
            .iter_mut()
            .zip(other.data.iter().chain(std::iter::repeat(&0)))
        {
            let old_v = *self_blk;
            let new_v = old_v & *other_blk;
            *self_blk = new_v;
            chngd |= old_v != new_v;
        }
        // We don't need to mask the last block as those bits can't be set by "&" by definition.
        chngd
    }

    /// We guarantee that the last storage block has no bits set past the "last" bit: this function
    /// clears any such bits.
    fn mask_last_block(&mut self) {
        debug_assert_eq!(self.data.len(), blocks_required(self.len));
        let ub = self.len % BITS_PER_BLOCK;
        // If there are no unused bits, there's no need to perform masking.
        if ub > 0 {
            let msk = (1 << ub) - 1;
            let off = block_offset(self.len);
            let old_v = self.data[off];
            let new_v = old_v & msk;
            if new_v != old_v {
                self.data[off] = new_v;
            }
        }
    }
}

impl Iterator for IterOnes<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bv.len {
            return None;
        }

        // start at current index, mask off earlier bits in the starting block
        let mut b = self.index / BITS_PER_BLOCK;
        let off = self.index % BITS_PER_BLOCK;

        // guard against empty storage
        if b >= self.bv.data.len() {
            self.index = self.bv.len;
            return None;
        }

        let mut v = self.bv.data[b];
        if off != 0 {
            v &= usize::MAX << off;
        }

        loop {
            if v != 0 {
                let tz = v.trailing_zeros() as usize;
                let bit = b * BITS_PER_BLOCK + tz;
                if bit < self.bv.len {
                    self.index = bit + 1;
                    return Some(bit);
                } else {
                    self.index = self.bv.len;
                    return None;
                }
            }

            b += 1;
            if b >= self.bv.data.len() {
                self.index = self.bv.len;
                return None;
            }
            v = self.bv.data[b];
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // cannot know remaining ones cheaply, use a safe upper bound
        let remaining = self.bv.len.saturating_sub(self.index);
        (0, Some(remaining))
    }
}

impl std::ops::BitOrAssign<&Self> for BitVec {
    #[inline(always)]
    fn bitor_assign(&mut self, other: &Self) {
        let _ = self.or(other);
    }
}

impl std::ops::BitOr<&BitVec> for &BitVec {
    type Output = BitVec;
    #[inline(always)]
    fn bitor(self, other: &BitVec) -> BitVec {
        let mut rv = self.clone();
        let _ = rv.or(other);
        rv
    }
}

impl std::ops::BitOr<&Self> for BitVec {
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, other: &Self) -> Self {
        let _ = self.or(other);
        self
    }
}

impl std::ops::BitAndAssign<&Self> for BitVec {
    #[inline(always)]
    fn bitand_assign(&mut self, other: &Self) {
        let _ = self.and(other);
    }
}

impl std::ops::BitAnd<&BitVec> for &BitVec {
    type Output = BitVec;
    #[inline(always)]
    fn bitand(self, other: &BitVec) -> BitVec {
        let mut rv = self.clone();
        let _ = rv.and(other);
        rv
    }
}

impl std::ops::BitAnd<&Self> for BitVec {
    type Output = Self;
    #[inline(always)]
    fn bitand(mut self, other: &Self) -> Self {
        let _ = self.and(other);
        self
    }
}

const BYTES_PER_BLOCK: usize = size_of::<usize>();
const BITS_PER_BLOCK: usize = BYTES_PER_BLOCK * 8;

#[inline(always)]
/// Takes as input a number of bits requiring storage; returns an aligned number of blocks needed
/// to store those bits.
const fn blocks_required(num_bits: usize) -> usize {
    let n = num_bits / BITS_PER_BLOCK;
    if num_bits % BITS_PER_BLOCK != 0 {
        n + 1
    } else {
        n
    }
}

#[inline(always)]
/// Return the offset in the vector of the storage block storing the bit `off`.
const fn block_offset(off: usize) -> usize {
    off / BITS_PER_BLOCK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_adjusts_vec_correctly_one() {
        let mut v = BitVec::new();
        assert_eq!(v.data.len(), 0);
        v.push(false);
        assert_eq!(v.data.len(), 1);
    }

    fn random_bitvec(len: usize) -> BitVec {
        let mut vob = BitVec::with_capacity(len);
        for _ in 0..len {
            vob.push(rand::random());
        }
        // these tests can later be dialed down, as they noticeable slow down every random vob test.
        assert_eq!(
            vob.iter_ones().count(),
            vob.iter_ones().filter(|_| true).count()
        );
        vob
    }

    #[test]
    fn test_count() {
        for test_len in 1..128 {
            let _ = random_bitvec(test_len);
        }
    }

    #[test]
    fn test_iter_ones() {
        #[allow(clippy::needless_pass_by_value)]
        fn t(v: &BitVec, expected: Vec<usize>) {
            assert_eq!(v.iter_ones().collect::<Vec<usize>>(), expected);
        }

        t(&BitVec::repeat(true, 131), (0..131).collect::<Vec<_>>());

        let mut v1 = BitVec::new();

        v1.push(false);
        v1.push(true);
        v1.push(false);
        v1.push(true);

        t(&v1, vec![1, 3]);
    }
}

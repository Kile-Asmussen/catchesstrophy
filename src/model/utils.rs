use std::{clone, iter::Map, ops::Deref};

#[inline]
pub fn bitor_sum<const N: usize>(data: &[u64; N]) -> u64 {
    let mut res = 0;
    for i in 0..N {
        res |= data[i];
    }
    res
}

pub trait IteratorExtensions: Iterator + Sized {
    fn clones<T>(self) -> impl Iterator<Item = T>
    where
        Self::Item: Deref<Target = T>,
        T: Clone,
    {
        self.map(|x| x.clone())
    }
}

impl<I: Iterator> IteratorExtensions for I {}

pub trait SliceExtensions<T>: Deref<Target = [T]> {
    fn clones<'a>(&'a self) -> impl Iterator<Item = T>
    where
        T: Clone + 'a,
    {
        self.iter().clones()
    }
}

impl<T, S: Deref<Target = [T]>> SliceExtensions<T> for S {}

#[macro_export]
macro_rules! biterate {
    {for $sq:ident in $mask:expr; $body:tt } => {{
        let mut mask: u64 = $mask;
        for _ in 0..=mask.count_ones() {
            let ix = mask.trailing_zeros();
            mask = !(1 << ix);
            let $sq = crate::model::Square::from_u8(ix as u8);
            $body
        }
    }};
}

pub use biterate;

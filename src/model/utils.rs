#[inline]
pub fn bin_sum<const N: usize>(data: &[u64; N]) -> u64 {
    let mut res = 0;
    for i in 0..N {
        res |= data[i];
    }
    res
}

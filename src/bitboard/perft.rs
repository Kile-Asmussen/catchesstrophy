use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

use rand::{RngCore, rngs::SmallRng};

use crate::{
    bitboard::{
        board::BitBoard,
        hash::{ZobHasher, ZobristTables, pi_rng},
        movegen::{BlessingStrategy, enumerate},
        moving::{clone_make_legal_move, make_legal_move, unmake_legal_move},
        utils::SliceExtensions,
        vision::Panopticon,
    },
    model::{LegalMove, Transients},
    notation::CoordNotation,
};

pub fn perft<
    BB: BitBoard,
    X: Panopticon,
    L: BlessingStrategy<Blessing = LegalMove>,
    RC: RecursionStrategy,
    ZT: ZobristTables,
>(
    depth: usize,
    bulk: bool,
    mut memoizer: impl PerftMemoizer,
) -> PerfTestRes {
    let mut breakdown = BTreeMap::new();
    let now = Instant::now();

    let mut firstmoves = vec![];
    let mut startpos = BB::startpos::<ZT>();

    if depth != 0 {
        enumerate::<BB, X, L>(&startpos, &mut firstmoves);

        if depth == 1 {
            for mv in firstmoves {
                let rec = RC::recurse::<BB, ZT>(&mut startpos, mv);
                breakdown.insert(CoordNotation::from(mv.0), 1);
                RC::reclaim::<BB, ZT>(rec);
            }
        } else {
            let mut buf = Vec::with_capacity(firstmoves.len());
            for mv in firstmoves {
                let mut rec = RC::recurse::<BB, ZT>(&mut startpos, mv);
                enumerate::<BB, X, L>(&mut *rec, &mut buf);
                breakdown.insert(
                    mv.0.into(),
                    perft_recurse::<BB, X, L, RC, ZT>(
                        depth - 1,
                        &mut *rec,
                        &buf[..],
                        bulk,
                        &mut memoizer,
                    ),
                );
                RC::reclaim::<BB, ZT>(rec);
            }
        }
    }

    PerfTestRes {
        elapsed_duration: now.elapsed(),
        breakdown,
        depth,
        memo_used: memoizer.size(),
    }
}

fn perft_recurse<
    BB: BitBoard,
    X: Panopticon,
    L: BlessingStrategy<Blessing = LegalMove>,
    RC: RecursionStrategy,
    ZT: ZobristTables,
>(
    depth: usize,
    board: &mut BB,
    moves: &[LegalMove],
    bulk: bool,
    mut memoizer: &mut impl PerftMemoizer,
) -> usize {
    if let Some(n) = memoizer.remember(board.curr_hash(), depth) {
        return n;
    }

    let mut res = 0;
    if depth == 0 {
        res += 1;
    } else if depth == 1 {
        if bulk {
            res += moves.len()
        } else {
            for mv in moves.clones() {
                let rec = RC::recurse::<BB, ZT>(board, mv);
                res += 1;
                RC::reclaim::<BB, ZT>(rec);
            }
        }
    } else {
        let mut buf = Vec::with_capacity(moves.len());
        for mv in moves.clones() {
            let mut rec = RC::recurse::<BB, ZT>(board, mv);
            if let Some(n) = memoizer.remember(rec.curr_hash(), depth - 1) {
                res += n;
            } else {
                enumerate::<BB, X, L>(&mut *rec, &mut buf);
                let n = perft_recurse::<BB, X, L, RC, ZT>(
                    depth - 1,
                    &mut *rec,
                    &buf[..],
                    bulk,
                    memoizer,
                );
                memoizer.memoize(rec.curr_hash(), depth - 1, n);
                res += n;
            }
            RC::reclaim::<BB, ZT>(rec);
        }
    }

    memoizer.memoize(board.curr_hash(), depth, res);

    res
}

pub struct PerfTestRes {
    pub depth: usize,
    pub elapsed_duration: Duration,
    pub breakdown: BTreeMap<CoordNotation, usize>,
    pub memo_used: (usize, usize),
}

impl PerfTestRes {
    pub fn pretty_print(&self) {
        println!("Performance test depth {}", self.depth);
        for (mv, n) in &self.breakdown {
            println!("{}: {}", mv, n);
        }
        println!(
            "Time elapsed: {:.02}ms",
            self.elapsed_duration.as_millis_f64()
        );
        println!(
            "Nodes per second: {:.02}",
            self.breakdown.values().sum::<usize>() as f64 / self.elapsed_duration.as_secs_f64()
        );
        println!("Memorization: {}/{}", self.memo_used.0, self.memo_used.1);
        println!("Nodes searched: {}", self.breakdown.values().sum::<usize>());
    }
}

pub trait PerftMemoizer {
    fn memoize(&mut self, key: u64, depth: usize, value: usize);
    fn remember(&self, key: u64, depth: usize) -> Option<usize>;
    fn size(&self) -> (usize, usize);
}

impl PerftMemoizer for () {
    #[inline]
    fn memoize(&mut self, key: u64, depth: usize, value: usize) {}

    #[inline]
    fn remember(&self, key: u64, depth: usize) -> Option<usize> {
        None
    }

    fn size(&self) -> (usize, usize) {
        (0, 0)
    }
}

pub struct HashMapMemo(HashMap<(u64, u64), usize, ZobHasher>, Vec<u64>, SmallRng);

impl HashMapMemo {
    pub fn new(depth: usize) -> Self {
        let mut rng = pi_rng();
        for _ in 0..10_000 {
            rng.next_u64();
        }
        let mut vec = Vec::with_capacity(depth);
        for _ in 0..depth {
            vec.push(rng.next_u64());
        }
        Self(
            HashMap::with_capacity_and_hasher(10usize.pow(depth as u32), ZobHasher(0)),
            vec,
            rng,
        )
    }
}

impl PerftMemoizer for HashMapMemo {
    fn memoize(&mut self, key: u64, depth: usize, value: usize) {
        while self.1.len() <= depth {
            self.1.push(self.2.next_u64())
        }
        self.0.insert((key, self.1[depth]), value);
    }

    fn remember(&self, key: u64, depth: usize) -> Option<usize> {
        if depth >= self.1.len() {
            return None;
        }
        self.0.get(&(key, self.1[depth])).map(|x| *x)
    }

    fn size(&self) -> (usize, usize) {
        (self.0.len(), self.0.capacity())
    }
}

pub trait RecursionStrategy {
    type Claim<'a, BB: BitBoard + 'a>: DerefMut<Target = BB>;
    fn recurse<'a, BB: BitBoard + 'a, ZT: ZobristTables>(
        board: &'a mut BB,
        mv: LegalMove,
    ) -> Self::Claim<'a, BB>;
    fn reclaim<'a, BB: BitBoard + 'a, ZT: ZobristTables>(claim: Self::Claim<'a, BB>);
}

pub struct MakeUnmake;
pub struct UnmakeClaim<'a, BB: BitBoard>(&'a mut BB, LegalMove, Transients);
impl<'a, BB: BitBoard> Deref for UnmakeClaim<'a, BB> {
    type Target = BB;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, BB: BitBoard> DerefMut for UnmakeClaim<'a, BB> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl RecursionStrategy for MakeUnmake {
    type Claim<'a, BB: BitBoard + 'a> = UnmakeClaim<'a, BB>;

    #[inline]
    fn recurse<'a, BB: BitBoard + 'a, ZT: ZobristTables>(
        board: &'a mut BB,
        mv: LegalMove,
    ) -> Self::Claim<'a, BB> {
        let trans = make_legal_move::<BB, ZT>(board, mv);
        UnmakeClaim(board, mv, trans)
    }

    #[inline]
    fn reclaim<'a, BB: BitBoard + 'a, ZT: ZobristTables>(claim: Self::Claim<'a, BB>) {
        unmake_legal_move::<BB, ZT>(claim.0, claim.1, claim.2);
    }
}

pub struct CloneMake;
pub struct DiscardCopyClaim<BB: BitBoard>(BB);
impl<BB: BitBoard> Deref for DiscardCopyClaim<BB> {
    type Target = BB;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<BB: BitBoard> DerefMut for DiscardCopyClaim<BB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RecursionStrategy for CloneMake {
    type Claim<'a, BB: BitBoard + 'a> = DiscardCopyClaim<BB>;

    #[inline]
    fn recurse<'a, BB: BitBoard + 'a, ZT: ZobristTables>(
        board: &'a mut BB,
        mv: LegalMove,
    ) -> Self::Claim<'a, BB> {
        DiscardCopyClaim(clone_make_legal_move::<BB, ZT>(board, mv))
    }

    #[inline]
    fn reclaim<'a, BB: BitBoard + 'a, ZT: ZobristTables>(claim: Self::Claim<'a, BB>) {}
}

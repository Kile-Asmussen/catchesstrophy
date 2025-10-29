use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    time::{Duration, Instant},
};

use crate::model::{
    LegalMove, Transients,
    bitboard::BitBoard,
    hash::ZobristTables,
    movegen::{BlessingStrategy, enumerate},
    moving::{clone_make_legal_move, make_legal_move, unmake_legal_move},
    notation::CoordNotation,
    utils::SliceExtensions,
    vision::Panopticon,
};

pub fn perft<
    BB: BitBoard,
    X: Panopticon,
    L: BlessingStrategy<Blessing = LegalMove>,
    RC: RecursionStrategy,
    ZT: ZobristTables,
>(
    depth: usize,
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
                buf.clear();
                let mut rec = RC::recurse::<BB, ZT>(&mut startpos, mv);
                enumerate::<BB, X, L>(&mut *rec, &mut buf);
                breakdown.insert(
                    CoordNotation::from(mv.0),
                    perft_recurse::<BB, X, L, RC, ZT>(depth - 1, &mut *rec, &buf[..]),
                );
                RC::reclaim::<BB, ZT>(rec);
            }
        }
    }

    PerfTestRes {
        elapsed_duration: now.elapsed(),
        breakdown,
        depth,
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
) -> usize {
    let mut res = 0;
    if depth == 0 {
        res += 1;
    } else if depth == 1 {
        for mv in moves.clones() {
            let rec = RC::recurse::<BB, ZT>(board, mv);
            res += 1;
            RC::reclaim::<BB, ZT>(rec);
        }
    } else {
        let mut buf = Vec::with_capacity(moves.len());
        for mv in moves.clones() {
            buf.clear();
            let mut rec = RC::recurse::<BB, ZT>(board, mv);
            enumerate::<BB, X, L>(&mut *rec, &mut buf);
            res += perft_recurse::<BB, X, L, RC, ZT>(depth - 1, &mut *rec, &buf[..]);
            RC::reclaim::<BB, ZT>(rec);
        }
    }

    res
}

pub struct PerfTestRes {
    pub depth: usize,
    pub elapsed_duration: Duration,
    pub breakdown: BTreeMap<CoordNotation, usize>,
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
        println!("Nodes searched: {}", self.breakdown.values().sum::<usize>())
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

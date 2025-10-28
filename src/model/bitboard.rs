use crate::model::{
    CLASSIC_CASTLING, Castling, ChessMan, Color, Transients,
    hash::{ZobristTables, bin_sum},
};
use strum::VariantArray;

pub trait BitBoard: ChessBoard {
    fn xor(&mut self, color: Color, man: ChessMan, mask: u64);
}

pub trait ChessBoard: MetaBoard {
    fn startpos<ZT: ZobristTables>() -> Self;

    fn sanity_check<ZT: ZobristTables>(&self);

    fn rehash<ZT: ZobristTables>(&self) -> u64;
}

pub trait MetaBoard: Clone {
    fn trans_mut(&mut self) -> &mut Transients;
    fn trans(&self) -> Transients;

    fn current_hash(&self) -> u64;
    fn hash(&mut self, hash: u64);

    fn ply(&self) -> (Color, u16);
    fn next_ply(&mut self);
    fn prev_ply(&mut self);

    fn castling(&self) -> &'static Castling;
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultMetaBoard {
    pub castling: &'static Castling,
    pub hash: u64,
    pub turn: u16,
    pub player: Color,
    pub trans: Transients,
}

impl PartialEq for DefaultMetaBoard {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player
            && self.trans.en_passant == other.trans.en_passant
            && self.trans.rights == other.trans.rights
    }
}

impl MetaBoard for DefaultMetaBoard {
    #[inline]
    fn trans_mut(&mut self) -> &mut Transients {
        &mut self.trans
    }

    #[inline]
    fn castling(&self) -> &'static Castling {
        self.castling
    }

    #[inline]
    fn trans(&self) -> Transients {
        self.trans
    }

    #[inline]
    fn ply(&self) -> (Color, u16) {
        (self.player, self.turn)
    }

    #[inline]
    fn next_ply(&mut self) {
        self.player = self.player.opp();
        if self.player == Color::WHITE {
            self.turn += 1;
        }
    }

    #[inline]
    fn prev_ply(&mut self) {
        if self.player == Color::WHITE {
            self.turn -= 1;
        }
        self.player = self.player.opp();
    }

    #[inline]
    fn current_hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    fn hash(&mut self, hash: u64) {
        self.hash ^= hash;
    }
}

impl ChessBoard for DefaultMetaBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        let mut res = Self {
            castling: &CLASSIC_CASTLING,
            hash: 0,
            player: Color::WHITE,
            turn: 1,
            trans: Transients {
                en_passant: None,
                halfmove_clock: 0,
                rights: [[true; 2]; 2],
            },
        };
        res.hash = res.rehash::<ZT>();
        res
    }

    #[inline]
    #[cfg(test)]
    fn sanity_check<ZT: ZobristTables>(&self) {
        assert_eq!(self.current_hash(), self.rehash::<ZT>());
    }

    #[cfg(not(test))]
    #[inline]
    fn sanity_check<ZT: ZobristTables>(&self) {}

    #[inline]
    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        let mut res = 0;
        let zobristtable = ZT::static_table();
        res ^= zobristtable.black();
        res ^= zobristtable.hash_rights(self.trans.rights);
        res ^= zobristtable.hash_en_passant(self.trans.en_passant);
        res
    }
}

pub trait HasDefaultMetaBoard {
    fn metaboard(&self) -> &DefaultMetaBoard;
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard;
}

impl<BB: HasDefaultMetaBoard + Clone> MetaBoard for BB {
    #[inline]
    fn trans_mut(&mut self) -> &mut Transients {
        self.metaboard_mut().trans_mut()
    }

    #[inline]
    fn trans(&self) -> Transients {
        self.metaboard().trans()
    }

    #[inline]
    fn current_hash(&self) -> u64 {
        self.metaboard().current_hash()
    }

    #[inline]
    fn hash(&mut self, hash: u64) {
        self.metaboard_mut().hash(hash)
    }

    #[inline]
    fn ply(&self) -> (Color, u16) {
        self.metaboard().ply()
    }

    #[inline]
    fn next_ply(&mut self) {
        self.metaboard_mut().next_ply();
    }

    #[inline]
    fn prev_ply(&mut self) {
        self.metaboard_mut().prev_ply();
    }

    #[inline]
    fn castling(&self) -> &'static Castling {
        self.metaboard().castling()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompactBitBoard {
    pub men: [u64; 6],
    pub colors: [u64; 2],
    pub meta: DefaultMetaBoard,
}

impl BitBoard for CompactBitBoard {
    #[inline]
    fn xor(&mut self, color: Color, man: ChessMan, mask: u64) {
        self.men[man.ix()] ^= mask;
        self.colors[color.ix()] ^= mask;
    }
}

impl HasDefaultMetaBoard for CompactBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        &self.meta
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        &mut self.meta
    }
}

impl ChessBoard for CompactBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        let mut res = Self {
            men: [
                0x00FF_0000_0000_FF00,
                0x4200_0000_0000_0042,
                0x2400_0000_0000_0024,
                0x8100_0000_0000_0081,
                0x0800_0000_0000_0008,
                0x1000_0000_0000_0010,
            ],
            colors: [0x0000_0000_0000_FFFF, 0xFFFF_0000_0000_0000],
            meta: DefaultMetaBoard::startpos::<ZT>(),
        };
        res.meta.hash = res.rehash::<ZT>();
        res
    }

    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.metaboard().rehash::<ZT>() ^ ZT::static_table().hash_compact(&self.colors, &self.men)
    }

    #[cfg(test)]
    fn sanity_check<ZT: ZobristTables>(&self) {
        for p1 in ChessMan::VARIANTS {
            for p2 in ChessMan::VARIANTS {
                let (p1, p2) = (*p1, *p2);
                if p1 >= p2 {
                    continue;
                }

                assert_eq!(
                    self.men[p1.ix()] & self.men[p2.ix()],
                    0,
                    "{:?} and {:?} overlap",
                    p1,
                    p2
                );
            }
        }

        assert_eq!(
            self.colors[Color::WHITE.ix()] | self.colors[Color::BLACK.ix()],
            0,
            "white and black overlap",
        );

        let mut white = 0;
        let mut black = 0;
        let mut total = 0;

        for p in &self.men {
            let p = *p;
            white |= self.colors[Color::WHITE.ix()] & p;
            black |= self.colors[Color::BLACK.ix()] & p;
            total |= p;
        }

        assert_eq!(
            white,
            self.colors[Color::WHITE.ix()],
            "sum of white-masked pieces not equal to white"
        );

        assert_eq!(
            black,
            self.colors[Color::BLACK as usize],
            "disjunction of black-masked pieces not equal to black"
        );

        assert_eq!(
            total,
            white | black,
            "disjunction of pieces not equal to disjunction of colors"
        );

        assert_eq!(
            self.current_hash(),
            self.rehash::<ZT>(),
            "procedural hash mismatch"
        );
    }

    #[inline]
    #[cfg(not(test))]
    fn sanity_check<ZT: ZobristTables>(&self) {}
}

#[derive(Debug, Clone)]
pub struct FullBitBoard {
    masks: [[u64; 6]; 2],
    meta: DefaultMetaBoard,
}

impl BitBoard for FullBitBoard {
    #[inline]
    fn xor(&mut self, color: Color, man: ChessMan, mask: u64) {
        self.masks[color.ix()][man.ix()] ^= mask;
    }
}

impl HasDefaultMetaBoard for FullBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        &self.meta
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        &mut self.meta
    }
}

impl ChessBoard for FullBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        let white = [0xFF00, 0x42, 0x24, 0x81, 0x08, 0x10u64];
        let black = white.map(|m| m.swap_bytes());
        let mut res = Self {
            masks: [white, black],
            meta: DefaultMetaBoard::startpos::<ZT>(),
        };
        res.meta.hash = res.rehash::<ZT>();
        res
    }

    #[cfg(test)]
    fn sanity_check<ZT: ZobristTables>(&self) {
        for p1 in ChessMan::VARIANTS {
            for p2 in ChessMan::VARIANTS {
                for c1 in [Color::WHITE, Color::BLACK] {
                    for c2 in [Color::WHITE, Color::BLACK] {
                        let (p1, p2) = (*p1, *p2);
                        if (p1, c1) >= (p2, c2) {
                            continue;
                        }

                        assert_eq!(
                            self.masks[c1.ix()][p1.ix()] & self.masks[c2.ix()][p2.ix()],
                            0,
                            "{:?} {:?} and {:?} {:?} overlap",
                            c1,
                            p1,
                            c2,
                            p2
                        );
                    }
                }
            }
        }

        assert_eq!(self.metaboard().current_hash(), self.rehash::<ZT>());
    }

    #[inline]
    #[cfg(not(test))]
    fn sanity_check<ZT: ZobristTables>(&self) {}

    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.metaboard().rehash::<ZT>() ^ ZT::static_table().hash_full(&self.masks)
    }
}

#[derive(Debug, Clone)]
pub struct FullerBitBoard {
    pub bitboard: FullBitBoard,
    pub total: [u64; 2],
}

impl BitBoard for FullerBitBoard {
    #[inline]
    fn xor(&mut self, color: Color, man: ChessMan, mask: u64) {
        self.bitboard.xor(color, man, mask);
        self.total[color.ix()] ^= mask;
    }
}

impl HasDefaultMetaBoard for FullerBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        self.bitboard.metaboard()
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        self.bitboard.metaboard_mut()
    }
}

impl ChessBoard for FullerBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        Self {
            bitboard: FullBitBoard::startpos::<ZT>(),
            total: [0x0000_0000_0000_FF00, 0x00FF_0000_0000_0000],
        }
    }

    #[cfg(test)]
    fn sanity_check<ZT: ZobristTables>(&self) {
        assert_eq!(
            self.total[Color::WHITE.ix()],
            bin_sum(&self.bitboard.masks[Color::WHITE.ix()]),
            "white total is not sum of white pieces"
        );
        assert_eq!(
            self.total[Color::BLACK.ix()],
            bin_sum(&self.bitboard.masks[Color::BLACK.ix()]),
            "black total is not sum of black pieces"
        )
    }

    #[inline]
    #[cfg(not(test))]
    fn sanity_check<ZT: ZobristTables>(&self) {}

    #[inline]
    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.bitboard.rehash::<ZT>()
    }
}

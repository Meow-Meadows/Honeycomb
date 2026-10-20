use crate::board::{Board, Color, Move, Piece};

pub const SEE_PIECE_VALUES: [i32; 6] = [100, 300, 300, 500, 900, 20_000];

impl Board {
    pub fn see_ge(&self, mv: Move, threshold: i32) -> bool {
        let from = mv.from;
        let to = mv.to;

        let moving_piece = match self.piece_at(self.side_to_move, from) {
            Some(p) => p,
            None => return false,
        };

        let us = self.side_to_move;
        let mut stm = us.opposite();

        let victim_val = match self.piece_at(stm, to) {
            Some(p) => SEE_PIECE_VALUES[p as usize],
            None => {
                if mv.promotion.is_some() {
                    0
                } else {
                    return 0 >= threshold;
                }
            }
        };
        let mut balance = victim_val - threshold;
        if balance < 0 {
            return false;
        }
        balance -= SEE_PIECE_VALUES[moving_piece as usize];
        if balance >= 0 {
            return true;
        }

        let from_bb = 1u64 << from;
        let to_bb = 1u64 << to;
        let mut occupied = self.occupied_squares() ^ from_bb ^ to_bb;
        let mut attackers = self.attackers_to(to, occupied) & occupied;
        let mut next_victim = moving_piece;
        loop {
            let stm_attackers = attackers & self.occupied_by(stm);
            if stm_attackers == 0 {
                break;
            }
            next_victim = self.min_attacker(to, stm_attackers, &mut occupied, &mut attackers);

            stm = stm.opposite();
            balance = -balance - 1 - SEE_PIECE_VALUES[next_victim as usize];

            if balance >= 0 {
                // captured with king
                if next_victim == Piece::King && (attackers & self.occupied_by(stm)) != 0 {
                    stm = stm.opposite();
                }
                break;
            }

            if next_victim == Piece::King {
                break;
            }
        }

        us != stm
    }

    fn min_attacker(
        &self,
        to: u8,
        stm_attackers: u64,
        occupied: &mut u64,
        attackers: &mut u64,
    ) -> Piece {
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let piece_bb = stm_attackers
                & (self.bitboard(Color::White, piece) | self.bitboard(Color::Black, piece));
            if piece_bb != 0 {
                //least valuable attacker square
                let lsb = 1u64 << piece_bb.trailing_zeros();
                *occupied ^= lsb;

                if matches!(piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                    let bishops_queens = (self.bitboard(Color::White, Piece::Bishop)
                        | self.bitboard(Color::Black, Piece::Bishop)
                        | self.bitboard(Color::White, Piece::Queen)
                        | self.bitboard(Color::Black, Piece::Queen))
                        & *occupied;
                    *attackers |= self.bishop_attacks_from(to, *occupied) & bishops_queens;
                }

                if matches!(piece, Piece::Rook | Piece::Queen) {
                    let rooks_queens = (self.bitboard(Color::White, Piece::Rook)
                        | self.bitboard(Color::Black, Piece::Rook)
                        | self.bitboard(Color::White, Piece::Queen)
                        | self.bitboard(Color::Black, Piece::Queen))
                        & *occupied;
                    *attackers |= self.rook_attacks_from(to, *occupied) & rooks_queens;
                }

                *attackers &= *occupied;
                return piece;
            }
        }

        unreachable!("stm_attackers was non-zero but no piece matched");
    }

    pub fn attackers_to(&self, square: u8, occupied: u64) -> u64 {
        let square_bit = 1u64 << square;
        let mut attackers = 0u64;

        let not_a = 0xFEFEFEFEFEFEFEFEu64;
        let not_h = 0x7F7F7F7F7F7F7F7Fu64;

        let w_pawns = self.bitboard(Color::White, Piece::Pawn);
        let b_pawns = self.bitboard(Color::Black, Piece::Pawn);

        let w_pawn_attacks = (((square_bit & not_h) >> 9) | ((square_bit & not_a) >> 7)) & w_pawns;
        let b_pawn_attacks = (((square_bit & not_a) << 9) | ((square_bit & not_h) << 7)) & b_pawns;
        attackers |= w_pawn_attacks | b_pawn_attacks;

        let knights =
            self.bitboard(Color::White, Piece::Knight) | self.bitboard(Color::Black, Piece::Knight);
        let file = square as i8 % 8;
        let rank = square as i8 / 8;

        for (df, dr) in [
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ] {
            let f = file + df;
            let r = rank + dr;
            if (0..8).contains(&f) && (0..8).contains(&r) {
                let bit = 1u64 << (r * 8 + f);
                if knights & bit != 0 {
                    attackers |= bit;
                }
            }
        }

        let kings =
            self.bitboard(Color::White, Piece::King) | self.bitboard(Color::Black, Piece::King);
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let f = file + df;
                let r = rank + dr;
                if (0..8).contains(&f) && (0..8).contains(&r) {
                    let bit = 1u64 << (r * 8 + f);
                    if kings & bit != 0 {
                        attackers |= bit;
                    }
                }
            }
        }

        let bishops_queens = self.bitboard(Color::White, Piece::Bishop)
            | self.bitboard(Color::Black, Piece::Bishop)
            | self.bitboard(Color::White, Piece::Queen)
            | self.bitboard(Color::Black, Piece::Queen);

        let rooks_queens = self.bitboard(Color::White, Piece::Rook)
            | self.bitboard(Color::Black, Piece::Rook)
            | self.bitboard(Color::White, Piece::Queen)
            | self.bitboard(Color::Black, Piece::Queen);

        attackers |= self.bishop_attacks_from(square, occupied) & bishops_queens;
        attackers |= self.rook_attacks_from(square, occupied) & rooks_queens;

        attackers
    }
}

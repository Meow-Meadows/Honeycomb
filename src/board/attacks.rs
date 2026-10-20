use crate::board::{Board, Color, Piece};

impl Board {
    pub fn in_check(&self, color: Color) -> bool {
        let Some(king_square) = self.king_square(color) else {
            return false;
        };

        self.is_square_attacked(king_square, color.opposite())
    }

    pub fn is_square_attacked(&self, square: u8, attacker: Color) -> bool {
        let king_bit = 1u64 << square;
        let occupied = self.occupied_squares();

        let enemy_pawns = self.bitboard(attacker, Piece::Pawn);
        let enemy_knights = self.bitboard(attacker, Piece::Knight);
        let enemy_bishops = self.bitboard(attacker, Piece::Bishop);
        let enemy_rooks = self.bitboard(attacker, Piece::Rook);
        let enemy_queens = self.bitboard(attacker, Piece::Queen);
        let enemy_king = self.bitboard(attacker, Piece::King);

        // pawns
        let not_a = 0xFEFEFEFEFEFEFEFEu64; // every square except file a
        let not_h = 0x7F7F7F7F7F7F7F7Fu64; // every square except file h

        let pawn_attacks = match attacker {
            Color::White => ((enemy_pawns & not_a) << 7) | ((enemy_pawns & not_h) << 9),
            Color::Black => ((enemy_pawns & not_h) >> 7) | ((enemy_pawns & not_a) >> 9),
        };

        if pawn_attacks & king_bit != 0 {
            return true;
        }

        let file = square as i8 % 8;
        let rank = square as i8 / 8;

        // knights
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
                if enemy_knights & bit != 0 {
                    return true;
                }
            }
        }

        // enemy king
        for df in -1..=1 {
            for dr in -1..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }

                let f = file + df;
                let r = rank + dr;

                if (0..8).contains(&f)
                    && (0..8).contains(&r)
                    && enemy_king & (1u64 << (r * 8 + f)) != 0
                {
                    return true;
                }
            }
        }

        // first piece found on each ray either attacks king or blocks ray
        let ray_attacked = |df: i8, dr: i8, attackers: u64| {
            let mut f = file + df;
            let mut r = rank + dr;

            while (0..8).contains(&f) && (0..8).contains(&r) {
                let bit = 1u64 << (r * 8 + f);

                if occupied & bit != 0 {
                    return attackers & bit != 0;
                }

                f += df;
                r += dr;
            }

            false
        };

        let diagonal_attackers = enemy_bishops | enemy_queens;
        for (df, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            if ray_attacked(df, dr, diagonal_attackers) {
                return true;
            }
        }

        let straight_attackers = enemy_rooks | enemy_queens;
        for (df, dr) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            if ray_attacked(df, dr, straight_attackers) {
                return true;
            }
        }

        false
    }
    pub fn bishop_attacks_from(&self, square: u8, occupied: u64) -> u64 {
        let mut attacks = 0u64;
        let file = square as i8 % 8;
        let rank = square as i8 / 8;

        for &(df, dr) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let mut f = file + df;
            let mut r = rank + dr;

            while (0..8).contains(&f) && (0..8).contains(&r) {
                let to = (r * 8 + f) as u8;
                let bit = 1u64 << to;
                attacks |= bit;

                if occupied & bit != 0 {
                    break;
                }

                f += df;
                r += dr;
            }
        }

        attacks
    }
    pub fn rook_attacks_from(&self, square: u8, occupied: u64) -> u64 {
        let mut attacks = 0u64;
        let file = square as i8 % 8;
        let rank = square as i8 / 8;

        for &(df, dr) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut f = file + df;
            let mut r = rank + dr;

            while (0..8).contains(&f) && (0..8).contains(&r) {
                let to = (r * 8 + f) as u8;
                let bit = 1u64 << to;
                attacks |= bit;

                if occupied & bit != 0 {
                    break;
                }

                f += df;
                r += dr;
            }
        }

        attacks
    }
}

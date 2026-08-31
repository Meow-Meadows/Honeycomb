#[derive(Clone)]
pub struct Board {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_queens: u64,
    pub white_king: u64,

    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_queens: u64,
    pub black_king: u64,

    pub white_to_move: bool,
}

#[derive(Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<char>,
}

impl Board {
    pub fn starting_position() -> Self {
        Board {
            white_pawns: 0x000000000000FF00,
            white_knights: 0x0000000000000042,
            white_bishops: 0x0000000000000024,
            white_rooks: 0x0000000000000081,
            white_queens: 0x0000000000000008,
            white_king: 0x0000000000000010,

            black_pawns: 0x00FF000000000000,
            black_knights: 0x4200000000000000,
            black_bishops: 0x2400000000000000,
            black_rooks: 0x8100000000000000,
            black_queens: 0x0800000000000000,
            black_king: 0x1000000000000000,

            white_to_move: true,
        }
    }

    pub fn occupied_squares(&self) -> u64 {
        self.white_pawns | self.white_knights | self.white_bishops
            | self.white_rooks | self.white_queens | self.white_king
            | self.black_pawns | self.black_knights | self.black_bishops
            | self.black_rooks | self.black_queens | self.black_king
    }

    pub fn empty_squares(&self) -> u64 {
        !self.occupied_squares()
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        let temp_legal_moves = self.generate_temp_legal_moves();

        for i in temp_legal_moves {
            let mut temp_board = self.clone();
            temp_board.make_move(i);
            if !temp_board.in_check(self.white_to_move) {
                legal_moves.push(i);
            }
        }
        legal_moves
    }

    fn pawn_moves(&self, moves: &mut Vec<Move>, mut dest: u64, shift: i8) {
        while dest != 0 {
            let to = dest.trailing_zeros() as u8;
            let from = (to as i8 - shift) as u8;

            let rank = to / 8;
            if rank == 0 || rank == 7 {
                moves.push(Move{from, to, promotion: Some('q')});
                moves.push(Move{from, to, promotion: Some('b')});
                moves.push(Move{from, to, promotion: Some('n')});
                moves.push(Move{from, to, promotion: Some('r')});
            }
            else {
                moves.push(Move{from, to, promotion: None});
            }

            dest &= dest - 1;
        }
    }

    fn normal_moves(&self, moves: &mut Vec<Move>, mut dest: u64, shift: i8) {
        while dest != 0 {
            let to = dest.trailing_zeros() as u8;
            let from = (to as i8 - shift) as u8;
            moves.push(Move{from, to, promotion: None});
            dest &= dest - 1;
        }
    }

    fn slider_moves(
        moves: &mut Vec<Move>,
        mut pieces: u64,
        own: u64,
        enemy: u64,
        directions: &[(i8, i8)],
    ) {
        while pieces != 0 {
            let from = pieces.trailing_zeros() as i8;
            pieces &= pieces -1;

            let file = from % 8;
            let rank = from / 8;

            for &(df, dr) in directions {
                let mut f = file + df;
                let mut r = rank + dr;

                while (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as u8;
                    let bit = 1u64 << to;

                    if own & bit != 0 {
                        break;
                    }

                    moves.push(Move {
                        from: from as u8,
                        to,
                        promotion: None
                    });

                    if enemy & bit != 0 {
                        break;
                    }

                    f += df;
                    r += dr;
                }
            }
        }
    }

    pub fn generate_temp_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let white = self.white_pawns | self.white_knights | self.white_bishops | self.white_rooks | self.white_queens | self.white_king;
        let black = self.black_pawns | self.black_knights | self.black_bishops | self.black_rooks | self.black_queens | self.black_king;

        //pawns :3
        let not_a: u64 = 0xFEFEFEFEFEFEFEFE;
        let not_h: u64 = 0x7F7F7F7F7F7F7F7F;

        if self.white_to_move {
            let pawns = self.white_pawns;
            let single = (pawns << 8) & self.empty_squares();
            self.pawn_moves(&mut moves, single, 8);

            let double = (single << 8) & self.empty_squares() & 0x00000000FF000000;
            self.pawn_moves(&mut moves, double, 16);

            let capture_left = ((pawns & not_a) << 7) & black;
            self.pawn_moves(&mut moves, capture_left, 7);
            let capture_right = ((pawns & not_h) << 9) & black;
            self.pawn_moves(&mut moves, capture_right, 9);
        }
        else {
            let pawns = self.black_pawns;

            let single = (pawns >> 8) & self.empty_squares();
            self.pawn_moves(&mut moves, single, -8);

            let double = (single >> 8) & self.empty_squares() & 0x000000FF00000000;
            self.pawn_moves(&mut moves, double, -16);

            let capture_left = ((pawns & not_h) >> 7) & white;
            self.pawn_moves(&mut moves, capture_left, -7);
            let capture_right = ((pawns & not_a) >> 9) & white;
            self.pawn_moves(&mut moves, capture_right, -9);
        }

        //knights :3
        let not_ab: u64 = 0xFCFCFCFCFCFCFCFC;
        let not_gh: u64 = 0x3F3F3F3F3F3F3F3F;

        let (knights, own) = if self.white_to_move {
            (self.white_knights, white)
        }
        else {
            (self.black_knights, black)
        };
        let k1 = ((knights & not_a) << 15) & (!own);
        self.normal_moves(&mut moves, k1, 15);

        let k2 = ((knights & not_h) << 17) & (!own);
        self.normal_moves(&mut moves, k2, 17);

        let k3 = ((knights & not_ab) << 6) & (!own);
        self.normal_moves(&mut moves, k3, 6);

        let k4 = ((knights & not_gh) << 10) & (!own);
        self.normal_moves(&mut moves, k4, 10);

        let k5 = ((knights & not_h) >> 15) & (!own);
        self.normal_moves(&mut moves, k5, -15);

        let k6 = ((knights & not_a) >> 17) & (!own);
        self.normal_moves(&mut moves, k6, -17);

        let k7 = ((knights & not_gh) >> 6) & (!own);
        self.normal_moves(&mut moves, k7, -6);

        let k8 = ((knights & not_ab) >> 10) & (!own);
        self.normal_moves(&mut moves, k8, -10);

        let (bishops, rooks, queens, king, own, enemy) = if self.white_to_move {
            (
                self.white_bishops,
                self.white_rooks,
                self.white_queens,
                self.white_king,
                white,
                black,
            )
        } else {
            (
                self.black_bishops,
                self.black_rooks,
                self.black_queens,
                self.black_king,
                black,
                white,
            )
        };

        let diagonal_directions = [
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let straight_directions = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
        ];

        let queen_directions = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        Self::slider_moves(
            &mut moves,
            bishops,
            own,
            enemy,
            &diagonal_directions,
        );

        Self::slider_moves(
            &mut moves,
            rooks,
            own,
            enemy,
            &straight_directions,
        );

        Self::slider_moves(
            &mut moves,
            queens,
            own,
            enemy,
            &queen_directions,
        );

        // king
        let from = king.trailing_zeros() as i8;
        let file = from % 8;
        let rank = from / 8;

        for (df, dr) in queen_directions {
            let f = file + df;
            let r = rank + dr;

            if (0..8).contains(&f) && (0..8).contains(&r) {
                let to = (r * 8 + f) as u8;
                let bit = 1u64 << to;

                if own & bit == 0 {
                    moves.push(Move {
                        from: from as u8,
                        to,
                        promotion: None,
                    })
                }
            }
        }

        moves
    }

    pub fn make_move(&mut self, i: Move) {
        let from = 1_u64 << i.from;
        let to = 1_u64 << i.to;

        //captures
        self.white_pawns &= !to;
        self.white_knights &= !to;
        self.white_bishops &= !to;
        self.white_rooks &= !to;
        self.white_queens &= !to;
        self.white_king &= !to;

        self.black_pawns &= !to;
        self.black_knights &= !to;
        self.black_bishops &= !to;
        self.black_rooks &= !to;
        self.black_queens &= !to;
        self.black_king &= !to;

        if (self.white_pawns & from) != 0 {
            self.white_pawns &= !from;
            self.white_pawns |= to;
        }
        else if (self.white_knights & from) != 0 {
            self.white_knights &= !from;
            self.white_knights |= to;
        }
        else if (self.white_bishops & from) != 0 {
            self.white_bishops &= !from;
            self.white_bishops |= to;
        }
        else if (self.white_rooks & from) != 0 {
            self.white_rooks &= !from;
            self.white_rooks |= to;
        }
        else if (self.white_queens & from) != 0 {
            self.white_queens &= !from;
            self.white_queens |= to;
        }
        else if (self.white_king & from) != 0 {
            self.white_king &= !from;
            self.white_king |= to;
        }

        else if (self.black_pawns & from) != 0 {
            self.black_pawns &= !from;
            self.black_pawns |= to;
        }
        else if (self.black_knights & from) != 0 {
            self.black_knights &= !from;
            self.black_knights |= to;
        }
        else if (self.black_bishops & from) != 0 {
            self.black_bishops &= !from;
            self.black_bishops |= to;
        }
        else if (self.black_rooks & from) != 0 {
            self.black_rooks &= !from;
            self.black_rooks |= to;
        }
        else if (self.black_queens & from) != 0 {
            self.black_queens &= !from;
            self.black_queens |= to;
        }
        else if (self.black_king & from) != 0 {
            self.black_king &= !from;
            self.black_king |= to;
        }

        self.white_to_move = !self.white_to_move;
    }

    pub fn in_check(&self, white: bool) -> bool {
        let king = if white { self.white_king } else { self.black_king };
        if king == 0 {
            return false;
        }

        let (enemy_pawns, enemy_knights, enemy_bishops, enemy_rooks, enemy_queens, enemy_king) =
            if white {
                (
                    self.black_pawns,
                    self.black_knights,
                    self.black_bishops,
                    self.black_rooks,
                    self.black_queens,
                    self.black_king,
                )
            } else {
                (
                    self.white_pawns,
                    self.white_knights,
                    self.white_bishops,
                    self.white_rooks,
                    self.white_queens,
                    self.white_king,
                )
            };

        let square = king.trailing_zeros() as i8;
        let file = square % 8;
        let rank = square / 8;
        let occupied = self.occupied_squares();

        // knights
        for (df, dr) in [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2),
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
                    continue
                }

                let f = file + df;
                let r = rank + dr;

                if (0..8).contains(&f) && (0..8).contains(&r) {
                    if enemy_king & (1u64 << (r * 8 + f)) != 0 {
                        return true;
                    }
                }
            }
        }

        // first piece found on each ray either attacks king or blocks ray
        let ray_attacked = |df : i8, dr : i8, attackers: u64| {
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
                return true
            }
        }

        // pawns
        let not_a = 0xFEFEFEFEFEFEFEFEu64; // every square except file a
        let not_h = 0x7F7F7F7F7F7F7F7Fu64; // every square except file h

        let pawn_attacks = if white {
            ((enemy_pawns & not_h) >> 7) | ((enemy_pawns & not_a) >> 9)
        } else {
            ((enemy_pawns & not_a) << 7) | ((enemy_pawns & not_h) << 9)
        };

        if pawn_attacks & king != 0 {
            return true;
        }

        false
    }
}
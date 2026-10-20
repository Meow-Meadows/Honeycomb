use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Color, Move, Piece, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};

impl Board {
    fn push_pawn_move(moves: &mut Vec<Move>, from: u8, to: u8) {
        let rank = to / 8;

        if rank == 0 || rank == 7 {
            moves.push(Move {
                from,
                to,
                promotion: Some(Piece::Queen),
            });
            moves.push(Move {
                from,
                to,
                promotion: Some(Piece::Rook),
            });
            moves.push(Move {
                from,
                to,
                promotion: Some(Piece::Bishop),
            });
            moves.push(Move {
                from,
                to,
                promotion: Some(Piece::Knight),
            });
        } else {
            moves.push(Move {
                from,
                to,
                promotion: None,
            });
        }
    }

    fn slider_moves(
        moves: &mut Vec<Move>,
        mut pieces: u64,
        own: u64,
        enemy: u64,
        enemy_king: u64,
        directions: &[(i8, i8)],
    ) {
        while pieces != 0 {
            let from = pieces.trailing_zeros() as u8;
            pieces &= pieces - 1;

            let file = from as i8 % 8;
            let rank = from as i8 / 8;

            for &(df, dr) in directions {
                let mut f = file + df;
                let mut r = rank + dr;

                while (0..8).contains(&r) && (0..8).contains(&f) {
                    let to = (r * 8 + f) as u8;
                    let bit = 1u64 << to;

                    if own & bit != 0 {
                        break;
                    }

                    // enemy king is never eaten
                    if enemy_king & bit != 0 {
                        break;
                    }

                    moves.push(Move {
                        from,
                        to,
                        promotion: None,
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

    fn add_castling_moves(&self, moves: &mut Vec<Move>) {
        let side = self.side_to_move;
        let enemy = side.opposite();
        let occupied = self.occupied_squares();

        if self.in_check(side) {
            return;
        }

        match side {
            Color::White => {
                // white kingside: e1 -> g1, rook h1 -> f1
                if self.castling_rights & WHITE_KINGSIDE != 0
                    && self.king_square(Color::White) == Some(4)
                    && self.bitboard(Color::White, Piece::Rook) & (1u64 << 7) != 0
                    && occupied & ((1u64 << 5) | (1u64 << 6)) == 0
                    && !self.is_square_attacked(5, enemy)
                    && !self.is_square_attacked(6, enemy)
                {
                    moves.push(Move {
                        from: 4,
                        to: 6,
                        promotion: None,
                    })
                }

                // white queenside: e1 -> c1, rook a1 -> d1
                if self.castling_rights & WHITE_QUEENSIDE != 0
                    && self.king_square(Color::White) == Some(4)
                    && self.bitboard(Color::White, Piece::Rook) & ((1u64) << 0) != 0
                    && occupied & ((1u64 << 1) | (1u64 << 2) | (1u64 << 3)) == 0
                    && !self.is_square_attacked(3, enemy)
                    && !self.is_square_attacked(2, enemy)
                {
                    moves.push(Move {
                        from: 4,
                        to: 2,
                        promotion: None,
                    })
                }
            }

            Color::Black => {
                // black kingside: e8 -> g8, rook h8 -> f8
                if self.castling_rights & BLACK_KINGSIDE != 0
                    && self.king_square(Color::Black) == Some(60)
                    && self.bitboard(Color::Black, Piece::Rook) & (1u64 << 63) != 0
                    && occupied & ((1u64 << 61) | (1u64 << 62)) == 0
                    && !self.is_square_attacked(61, enemy)
                    && !self.is_square_attacked(62, enemy)
                {
                    moves.push(Move {
                        from: 60,
                        to: 62,
                        promotion: None,
                    });
                }

                // black queenside: e8 -> c8, rook a8 -> d8
                if self.castling_rights & BLACK_QUEENSIDE != 0
                    && self.king_square(Color::Black) == Some(60)
                    && self.bitboard(Color::Black, Piece::Rook) & (1u64 << 56) != 0
                    && occupied & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59)) == 0
                    && !self.is_square_attacked(59, enemy)
                    && !self.is_square_attacked(58, enemy)
                {
                    moves.push(Move {
                        from: 60,
                        to: 58,
                        promotion: None,
                    });
                }
            }
        }
    }

    pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let side = self.side_to_move;
        let enemy_side = side.opposite();

        let own = self.occupied_by(side);
        let enemy = self.occupied_by(enemy_side);
        let enemy_king = self.bitboard(enemy_side, Piece::King);
        let occupied = own | enemy;

        // pawns
        let mut pawns = self.bitboard(side, Piece::Pawn);
        let direction: i8 = if side == Color::White { 1 } else { -1 };
        let start_rank: i8 = if side == Color::White { 1 } else { 6 };

        while pawns != 0 {
            let from = pawns.trailing_zeros() as u8;
            pawns &= pawns - 1;

            let file = from as i8 % 8;
            let rank = from as i8 / 8;

            let next_rank = rank + direction;

            // one square pawn push
            if (0..8).contains(&next_rank) {
                let to = (next_rank * 8 + file) as u8;
                let bit = 1u64 << to;

                if occupied & bit == 0 {
                    Self::push_pawn_move(&mut moves, from, to);

                    // two square pawn push
                    if rank == start_rank {
                        let double_rank = rank + 2 * direction;

                        if (0..8).contains(&double_rank) {
                            let double_to = (double_rank * 8 + file) as u8;
                            let double_bit = 1u64 << double_to;

                            if occupied & double_bit == 0 {
                                moves.push(Move {
                                    from,
                                    to: double_to,
                                    promotion: None,
                                });
                            }
                        }
                    }
                }
            }

            // pawn captures
            for df in [-1, 1] {
                let capture_file = file + df;
                let capture_rank = rank + direction;

                if (0..8).contains(&capture_file) && (0..8).contains(&capture_rank) {
                    let to = (capture_rank * 8 + capture_file) as u8;
                    let bit = 1u64 << to;

                    if enemy & bit != 0 && enemy_king & bit == 0 {
                        Self::push_pawn_move(&mut moves, from, to);
                    } else if self.en_passant == Some(to) {
                        let captured_pawn_square =
                            if side == Color::White { to - 8 } else { to + 8 };

                        let captured_pawn_bit = 1u64 << captured_pawn_square;

                        if self.bitboard(enemy_side, Piece::Pawn) & captured_pawn_bit != 0 {
                            moves.push(Move {
                                from,
                                to,
                                promotion: None,
                            });
                        }
                    }
                }
            }
        }

        // knights
        let mut knights = self.bitboard(side, Piece::Knight);

        while knights != 0 {
            let from = knights.trailing_zeros() as u8;
            knights &= knights - 1;

            let file = from as i8 % 8;
            let rank = from as i8 / 8;

            for (df, dr) in [
                (1, 2),
                (2, 1),
                (-1, 2),
                (2, -1),
                (-2, 1),
                (1, -2),
                (-1, -2),
                (-2, -1),
            ] {
                let f = file + df;
                let r = rank + dr;

                if (0..8).contains(&f) && (0..8).contains(&r) {
                    let to = (r * 8 + f) as u8;
                    let bit = 1u64 << to;

                    if own & bit == 0 && enemy_king & bit == 0 {
                        moves.push(Move {
                            from,
                            to,
                            promotion: None,
                        });
                    }
                }
            }
        }

        let diagonals = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        let straights = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let queen_directions = [
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
        ];

        // bishop
        Self::slider_moves(
            &mut moves,
            self.bitboard(side, Piece::Bishop),
            own,
            enemy,
            enemy_king,
            &diagonals,
        );

        // rook
        Self::slider_moves(
            &mut moves,
            self.bitboard(side, Piece::Rook),
            own,
            enemy,
            enemy_king,
            &straights,
        );

        // queen
        Self::slider_moves(
            &mut moves,
            self.bitboard(side, Piece::Queen),
            own,
            enemy,
            enemy_king,
            &queen_directions,
        );

        // king
        if let Some(from) = self.king_square(side) {
            let file = from as i8 % 8;
            let rank = from as i8 / 8;

            for (df, dr) in queen_directions {
                let f = file + df;
                let r = rank + dr;

                if (0..8).contains(&f) && (0..8).contains(&r) {
                    let to = (r * 8 + f) as u8;
                    let bit = 1u64 << to;

                    if own & bit == 0 && enemy_king & bit == 0 {
                        moves.push(Move {
                            from,
                            to,
                            promotion: None,
                        })
                    }
                }
            }
        }

        self.add_castling_moves(&mut moves);

        moves
    }

    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        let moving_side = self.side_to_move;
        let mut legal_moves = Vec::new();

        for mv in self.generate_pseudo_legal_moves() {
            let undo = self.make_move(mv);
            let is_legal = !self.in_check(moving_side);

            self.unmake_move(undo);

            if is_legal {
                legal_moves.push(mv);
            }
        }

        legal_moves
    }
    pub fn perft(&mut self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let moving_side = self.side_to_move;
        let moves = self.generate_pseudo_legal_moves();
        let mut nodes = 0;

        for mv in moves {
            let undo = self.make_move(mv);

            if !self.in_check(moving_side) {
                nodes += self.perft(depth - 1);
            }

            self.unmake_move(undo);
        }

        nodes
    }
}

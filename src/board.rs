#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promotion: Option<Piece>,
}

#[derive(Clone, Copy)]
pub struct Undo {
    mv: Move,
    moving_side: Color,
    moved_piece: Piece,
    captured: Option<(Piece, u8)>,

    old_castling_rights: u8,
    old_en_passant: Option<u8>,
    old_halfmove_clock: u32,
    old_fullmove_number: u32,
}

pub const WHITE_KINGSIDE: u8 = 1;
pub const WHITE_QUEENSIDE: u8 = 2;
pub const BLACK_KINGSIDE: u8 = 4;
pub const BLACK_QUEENSIDE: u8 = 8;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    // pieces[colour][piece]
    pieces: [[u64; 6]; 2],

    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pieces: [[0; 6]; 2],
            side_to_move: Color::White,
            castling_rights: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn starting_position() -> Self {
        Self {
            pieces: [
                [
                    0x000000000000FF00, // white pawns
                    0x0000000000000042, // white knights
                    0x0000000000000024, // white bishops
                    0x0000000000000081, // white rooks
                    0x0000000000000008, // white queen
                    0x0000000000000010, // white king
                ],
                [
                    0x00FF000000000000, // black pawns
                    0x4200000000000000, // black knights
                    0x2400000000000000, // black bishops
                    0x8100000000000000, // black rooks
                    0x0800000000000000, // black queen
                    0x1000000000000000, // black king
                ],
            ],

            side_to_move: Color::White,
            castling_rights: WHITE_KINGSIDE
                | WHITE_QUEENSIDE
                | BLACK_KINGSIDE
                | BLACK_QUEENSIDE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn bitboard(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }

    pub fn set_piece(&mut self, color: Color, piece: Piece, square: u8) {
        self.clear_square(square);
        self.add_piece(color, piece, square);
    }

    pub fn clear_square(&mut self, square: u8) {
        let bit = !(1u64 << square);

        for color in 0..2 {
            for piece in 0..6 {
                self.pieces[color][piece] &= bit;
            }
        }
    }

    fn add_piece(&mut self, color: Color, piece: Piece, square: u8) {
        self.pieces[color as usize][piece as usize] |= 1u64 << square;
    }

    fn remove_piece(&mut self, color: Color, piece: Piece, square: u8) {
        self.pieces[color as usize][piece as usize] &= !(1u64 << square);
    }

    pub fn occupied_by(&self, color: Color) -> u64 {
        self.pieces[color as usize]
            .iter()
            .fold(0, |occupied, &piece_board| occupied | piece_board)
    }

    pub fn occupied_squares(&self) -> u64 {
        self.occupied_by(Color::White) | self.occupied_by(Color::Black)
    }

    pub fn empty_squares(&self) -> u64 {
        !self.occupied_squares()
    }

    pub fn king_square(&self, color: Color) -> Option<u8> {
        let king = self.bitboard(color, Piece::King);

        if king == 0 {
            None
        } else {
            Some(king.trailing_zeros() as u8)
        }
    }

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
            Color::White => {
                ((enemy_pawns & not_a) << 7) | ((enemy_pawns & not_h) << 9)
            }
            Color::Black => {
                ((enemy_pawns & not_h) >> 7) | ((enemy_pawns & not_a) >> 9)
            }
        };

        if pawn_attacks & king_bit != 0 {
            return true;
        }

        let file = square as i8 % 8;
        let rank = square as i8 / 8;

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

        false
    }

    fn piece_at(&self, color: Color, square: u8) -> Option<Piece> {
        let bit = 1u64 << square;

        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            if self.bitboard(color, piece) & bit != 0 {
                return Some(piece);
            }
        }

        None
    }

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
            pieces &= pieces -1;

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
            pawns &= pawns -1;

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
                        let captured_pawn_square = if side == Color::White {
                            to - 8
                        } else {
                            to + 8
                        };

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
            knights &= knights -1;

            let file = from as i8 % 8;
            let rank = from as i8 / 8;

            for (df, dr) in [
                (1, 2), (2, 1), (-1, 2), (2, -1),
                (-2, 1), (1, -2), (-1, -2), (-2, -1),
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
            (1, 1), (1, -1), (-1, 1), (-1, -1),
            (1, 0), (-1, 0), (0, 1), (0, -1)
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

    pub fn make_move(&mut self, mv: Move) -> Undo {
        let moving_side = self.side_to_move;
        let enemy_side = moving_side.opposite();

        let moved_piece = self
            .piece_at(moving_side, mv.from)
            .expect("move must start on a piece belonging to the side to move");

        let is_en_passant = moved_piece == Piece::Pawn
            && self.en_passant == Some(mv.to)
            && mv.from % 8 != mv.to % 8
            && self.piece_at(enemy_side, mv.to).is_none();

        let is_castle = moved_piece == Piece::King
            && (mv.from as i8 - mv.to as i8).abs() == 2;

        let captured = if is_en_passant {
            let captured_square = if moving_side == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };

            Some((Piece::Pawn, captured_square))
        } else {
            self.piece_at(enemy_side, mv.to)
                .map(|piece| (piece, mv.to))
        };

        let undo = Undo {
            mv,
            moving_side,
            moved_piece,
            captured,
            old_castling_rights: self.castling_rights,
            old_en_passant: self.en_passant,
            old_halfmove_clock: self.halfmove_clock,
            old_fullmove_number: self.fullmove_number,
        };

        let is_capture = captured.is_some();

        if moved_piece == Piece::King {
            match moving_side {
                Color::White => {
                    self.castling_rights &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
                }

                Color::Black => {
                    self.castling_rights &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
                }
            }
        }

        if moved_piece == Piece::Rook {
            match mv.from {
                0 => self.castling_rights &= !WHITE_QUEENSIDE,
                7 => self.castling_rights &= !WHITE_KINGSIDE,
                56 => self.castling_rights &= !BLACK_QUEENSIDE,
                63 => self.castling_rights &= !BLACK_KINGSIDE,
                _ => {}
            }
        }

        match mv.to {
            0 => self.castling_rights &= !WHITE_QUEENSIDE,
            7 => self.castling_rights &= !WHITE_KINGSIDE,
            56 => self.castling_rights &= !BLACK_QUEENSIDE,
            63 => self.castling_rights &= !BLACK_KINGSIDE,
            _ => {}
        }

        self.remove_piece(moving_side, moved_piece, mv.from);

        if let Some((captured_piece, captured_square)) = captured {
            self.remove_piece(enemy_side, captured_piece, captured_square);
        }

        let piece_on_destination = mv.promotion.unwrap_or(moved_piece);

        self.add_piece(moving_side, piece_on_destination, mv.to);

        if is_castle {
            let (rook_from, rook_to) = match mv.to {
                6 => (7, 5),     // White kingside
                2 => (0, 3),     // White queenside
                62 => (63, 61),  // Black kingside
                58 => (56, 59),  // Black queenside
                _ => unreachable!("king moved two squares but was not castling"),
            };

            self.remove_piece(moving_side, Piece::Rook, rook_from);
            self.add_piece(moving_side, Piece::Rook, rook_to);
        }

        self.en_passant = None;

        if moved_piece == Piece::Pawn
            && (mv.from as i8 - mv.to as i8).abs() == 16
        {
            self.en_passant = Some((mv.to + mv.from) / 2);
        }

        if moved_piece == Piece::Pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if moving_side == Color::Black {
            self.fullmove_number += 1;
        }

        self.side_to_move = enemy_side;

        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        let mv = undo.mv;
        let moving_side = undo.moving_side;

        let is_castle = undo.moved_piece == Piece::King
            && (mv.from as i8 - mv.to as i8).abs() == 2;

        self.side_to_move = moving_side;
        self.castling_rights = undo.old_castling_rights;
        self.en_passant = undo.old_en_passant;
        self.halfmove_clock = undo.old_halfmove_clock;
        self.fullmove_number = undo.old_fullmove_number;

        let piece_on_destination = mv.promotion.unwrap_or(undo.moved_piece);
        self.remove_piece(moving_side, piece_on_destination, mv.to);
        self.add_piece(moving_side, undo.moved_piece, mv.from);

        if is_castle {
            let (rook_from, rook_to) = match mv.to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => unreachable!("king moved two squares but was not castling"),
            };

            self.remove_piece(moving_side, Piece::Rook, rook_to);
            self.add_piece(moving_side, Piece::Rook, rook_from);
        }

        if let Some((captured_piece, captured_square)) = undo.captured {
            self.add_piece(moving_side.opposite(), captured_piece, captured_square);
        }
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

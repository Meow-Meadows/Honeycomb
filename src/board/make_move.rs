use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, Board, Color, Move, Piece, Undo, WHITE_KINGSIDE,
    WHITE_QUEENSIDE,
};
use crate::zobrist;

impl Board {
    pub fn compute_hash(&self) -> u64 {
        let mut hash = 0u64;
        for color_idx in 0..2 {
            let color = if color_idx == 0 {
                Color::White
            } else {
                Color::Black
            };
            for piece_idx in 0..6 {
                let piece = match piece_idx {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    5 => Piece::King,
                    _ => unreachable!(),
                };

                let mut bb = self.pieces[color_idx][piece_idx];
                while bb != 0 {
                    let square = bb.trailing_zeros() as u8;
                    bb &= bb - 1;
                    hash ^= zobrist::piece_key(color, piece, square);
                }
            }
        }
        if self.side_to_move == Color::Black {
            hash ^= zobrist::side_key();
        }
        hash ^= zobrist::castling_key(self.castling_rights);
        if let Some(eq_sq) = self.en_passant {
            hash ^= zobrist::en_passant_key(eq_sq);
        }

        hash
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

        let is_castle = moved_piece == Piece::King && (mv.from as i8 - mv.to as i8).abs() == 2;

        let captured = if is_en_passant {
            let captured_square = if moving_side == Color::White {
                mv.to - 8
            } else {
                mv.to + 8
            };

            Some((Piece::Pawn, captured_square))
        } else {
            self.piece_at(enemy_side, mv.to).map(|piece| (piece, mv.to))
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
            old_hash: self.hash,
        };

        let is_capture = captured.is_some();

        if let Some(eq_sq) = self.en_passant {
            self.hash ^= zobrist::en_passant_key(eq_sq);
        }
        self.hash ^= zobrist::castling_key(self.castling_rights);

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

        self.hash ^= zobrist::castling_key(self.castling_rights);
        self.hash ^= zobrist::piece_key(moving_side, moved_piece, mv.from);

        self.remove_piece(moving_side, moved_piece, mv.from);

        if let Some((captured_piece, captured_square)) = captured {
            self.hash ^= zobrist::piece_key(enemy_side, captured_piece, captured_square);
            self.remove_piece(enemy_side, captured_piece, captured_square);
        }

        let piece_on_destination = mv.promotion.unwrap_or(moved_piece);
        self.hash ^= zobrist::piece_key(moving_side, piece_on_destination, mv.to);
        self.add_piece(moving_side, piece_on_destination, mv.to);

        if is_castle {
            let (rook_from, rook_to) = match mv.to {
                6 => (7, 5),    // White kingside
                2 => (0, 3),    // White queenside
                62 => (63, 61), // Black kingside
                58 => (56, 59), // Black queenside
                _ => unreachable!("king moved two squares but was not castling"),
            };

            self.hash ^= zobrist::piece_key(moving_side, Piece::Rook, rook_from);
            self.remove_piece(moving_side, Piece::Rook, rook_from);

            self.hash ^= zobrist::piece_key(moving_side, Piece::Rook, rook_to);
            self.add_piece(moving_side, Piece::Rook, rook_to);
        }

        self.en_passant = None;
        if moved_piece == Piece::Pawn && (mv.from as i8 - mv.to as i8).abs() == 16 {
            let ep_sq = (mv.to + mv.from) / 2;
            self.en_passant = Some(ep_sq);
            self.hash ^= zobrist::en_passant_key(ep_sq);
        }

        if moved_piece == Piece::Pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if moving_side == Color::Black {
            self.fullmove_number += 1;
        }

        self.hash ^= zobrist::side_key();
        self.side_to_move = enemy_side;

        undo
    }
    pub fn unmake_move(&mut self, undo: Undo) {
        let mv = undo.mv;
        let moving_side = undo.moving_side;
        let enemy_side = moving_side.opposite();

        self.side_to_move = moving_side;
        self.castling_rights = undo.old_castling_rights;
        self.en_passant = undo.old_en_passant;
        self.halfmove_clock = undo.old_halfmove_clock;
        self.fullmove_number = undo.old_fullmove_number;
        self.hash = undo.old_hash;

        let piece_on_destination = mv.promotion.unwrap_or(undo.moved_piece);
        self.remove_piece(moving_side, piece_on_destination, mv.to);
        self.add_piece(moving_side, undo.moved_piece, mv.from);

        if let Some((captured_piece, captured_square)) = undo.captured {
            self.add_piece(enemy_side, captured_piece, captured_square);
        }

        let is_castle = undo.moved_piece == Piece::King && (mv.from as i8 - mv.to as i8).abs() == 2;
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
    }
}

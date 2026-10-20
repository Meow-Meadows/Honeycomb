pub mod attacks;
pub mod make_move;
pub mod moves;
pub mod see;
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
    old_hash: u64,
}

pub const WHITE_KINGSIDE: u8 = 1;
pub const WHITE_QUEENSIDE: u8 = 2;
pub const BLACK_KINGSIDE: u8 = 4;
pub const BLACK_QUEENSIDE: u8 = 8;

#[derive(Clone, Debug, Eq)]
pub struct Board {
    // pieces[colour][piece]
    pieces: [[u64; 6]; 2],

    pub side_to_move: Color,
    pub castling_rights: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub hash: u64,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.side_to_move == other.side_to_move
            && self.castling_rights == other.castling_rights
            && self.en_passant == other.en_passant
            && self.halfmove_clock == other.halfmove_clock
            && self.fullmove_number == other.fullmove_number
    }
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
            hash: 0,
        }
    }

    pub fn starting_position() -> Self {
        let mut board = Self {
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
            castling_rights: WHITE_KINGSIDE | WHITE_QUEENSIDE | BLACK_KINGSIDE | BLACK_QUEENSIDE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
        };
        board.hash = board.compute_hash();
        board
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
    pub fn piece_at(&self, color: Color, square: u8) -> Option<Piece> {
        let bit = 1u64 << square;

        [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ]
        .into_iter()
        .find(|&piece| self.bitboard(color, piece) & bit != 0)
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut parts = fen.split_whitespace();
        let piece_placement = parts.next()?;
        let side_to_move = parts.next()?;
        let castling_rights = parts.next()?;
        let en_passant = parts.next()?;
        let halfmove_clock = parts.next().unwrap_or("0");
        let fullmove_number = parts.next().unwrap_or("1");

        let mut board = Self::empty();

        let mut rank = 7i8;
        let mut file = 0i8;

        for ch in piece_placement.chars() {
            match ch {
                '/' => {
                    if file != 8 || rank == 0 {
                        return None;
                    }
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += ch.to_digit(10)? as i8;
                    if file > 8 {
                        return None;
                    }
                }
                other => {
                    if rank < 0 || file > 7 {
                        return None;
                    }
                    let square = (rank * 8 + file) as u8;
                    let bit = 1u64 << square;

                    let (color_index, piece_index) = match other {
                        'P' => (0, 0),
                        'N' => (0, 1),
                        'B' => (0, 2),
                        'R' => (0, 3),
                        'Q' => (0, 4),
                        'K' => (0, 5),
                        'p' => (1, 0),
                        'n' => (1, 1),
                        'b' => (1, 2),
                        'r' => (1, 3),
                        'q' => (1, 4),
                        'k' => (1, 5),
                        _ => return None,
                    };

                    board.pieces[color_index][piece_index] |= bit;
                    file += 1;
                }
            }
        }

        if rank != 0 || file != 8 {
            return None;
        }

        board.side_to_move = match side_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };

        if castling_rights != "-" {
            for ch in castling_rights.chars() {
                match ch {
                    'K' => board.castling_rights |= WHITE_KINGSIDE,
                    'Q' => board.castling_rights |= WHITE_QUEENSIDE,
                    'k' => board.castling_rights |= BLACK_KINGSIDE,
                    'q' => board.castling_rights |= BLACK_QUEENSIDE,
                    _ => return None,
                }
            }
        }

        if en_passant == "-" {
            board.en_passant = None;
        } else {
            let bytes = en_passant.as_bytes();
            if bytes.len() != 2 {
                return None;
            }
            let file_letter = match bytes[0] {
                b'a'..=b'h' => bytes[0] - b'a',
                _ => return None,
            };
            let ep_rank = match bytes[1] {
                b'3' => 2,
                b'6' => 5,
                _ => return None,
            };
            board.en_passant = Some(ep_rank * 8 + file_letter);
        }

        board.halfmove_clock = halfmove_clock.parse().ok()?;
        board.fullmove_number = fullmove_number.parse().ok()?;
        board.hash = board.compute_hash();

        Some(board)
    }
}

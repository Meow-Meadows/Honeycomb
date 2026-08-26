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

        if self.white_to_move {
            let knights = self.white_knights;
            let own = white;
        }
        else {
            let knights = self.black_knights;
            let own = black;
        }
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

}

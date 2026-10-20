use crate::board::{Color, Piece};
use std::sync::LazyLock;
pub type ZobristKey = u64;
struct Prng {
    state: u64,
}
impl Prng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_int(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

#[derive(Debug, Clone)]
pub struct ZobristRandoms {
    ///[side][piece_type][square]
    pub pieces: [[[ZobristKey; 64]; 6]; 2],
    pub side_to_move: [ZobristKey; 2],
    pub castling: [ZobristKey; 16],
    pub en_passant: [ZobristKey; 65],
}
impl Default for ZobristRandoms {
    fn default() -> Self {
        Self::new()
    }
}
impl ZobristRandoms {
    const PRNG_SEED: u64 = 0x07072023;
    pub fn new() -> Self {
        Self::from_seed(Self::PRNG_SEED)
    }
    pub fn from_seed(seed: u64) -> Self {
        let mut prng = Prng::new(seed);

        let mut pieces = [[[0u64; 64]; 6]; 2];
        for side in &mut pieces {
            for piece in side {
                for sq in piece {
                    *sq = prng.next_int();
                }
            }
        }

        let mut castling = [0u64; 16];
        for entry in &mut castling {
            *entry = prng.next_int();
        }
        let mut side_to_move = [0u64; 2];
        side_to_move[0] = prng.next_int();
        side_to_move[1] = prng.next_int();
        let mut en_passant = [0u64; 65];
        for entry in &mut en_passant {
            *entry = prng.next_int();
        }
        Self {
            pieces,
            side_to_move,
            castling,
            en_passant,
        }
    }
    #[inline(always)]
    pub fn acc_piece(&self, side: usize, piece: usize, square: usize) -> ZobristKey {
        self.pieces[side][piece][square]
    }
    #[inline(always)]
    pub fn acc_castling(&self, castling_rights: usize) -> ZobristKey {
        self.castling[castling_rights]
    }
    #[inline(always)]
    pub fn acc_side(&self, side: usize) -> ZobristKey {
        self.side_to_move[side]
    }
    #[inline(always)]
    pub fn acc_en_passant(&self, square: Option<usize>) -> ZobristKey {
        match square {
            Some(sq) => self.en_passant[sq],
            None => self.en_passant[64],
        }
    }
}

static KEYS: LazyLock<ZobristRandoms> = LazyLock::new(ZobristRandoms::new);

#[inline(always)]
pub fn piece_key(color: Color, piece: Piece, square: u8) -> ZobristKey {
    KEYS.acc_piece(color as usize, piece as usize, square as usize)
}
#[inline(always)]
pub fn side_key() -> ZobristKey {
    KEYS.acc_side(1)
}
#[inline(always)]
pub fn castling_key(castling_rights: u8) -> ZobristKey {
    KEYS.acc_castling(castling_rights as usize)
}
#[inline(always)]
pub fn en_passant_key(square: u8) -> ZobristKey {
    KEYS.acc_en_passant(Some(square as usize))
}

use std::io::{self, BufRead, Write};

use crate::{
    board::{Board, Move, Piece},
    search::find_best_move,
};

fn square(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();

    if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1]) {
        return None;
    }

    Some((bytes[1] - b'1') * 8 + (bytes[0] - b'a'))
}

fn promotion(c: u8) -> Option<Piece> {
    match c {
        b'q' => Some(Piece::Queen),
        b'r' => Some(Piece::Rook),
        b'b' => Some(Piece::Bishop),
        b'n' => Some(Piece::Knight),
        _ => None,
    }
}

fn parse_move(board: &mut Board, text: &str) -> Option<Move> {
    let from = square(text.get(0..2)?)?;
    let to = square(text.get(2..4)?)?;
    let promotion = text.as_bytes().get(4).and_then(|&c| promotion(c));

    board
        .generate_legal_moves()
        .into_iter()
        .find(|mv| mv.from == from && mv.to == to && mv.promotion == promotion)
}

fn move_to_uci(mv: Move) -> String {
    let file = |sq: u8| (b'a' + sq % 8) as char;
    let rank = |sq: u8| (b'1' + sq / 8) as char;

    let mut result = format!(
        "{}{}{}{}",
        file(mv.from),
        rank(mv.from),
        file(mv.to),
        rank(mv.to),
    );

    if let Some(piece) = mv.promotion {
        result.push(match piece {
            Piece::Queen => 'q',
            Piece::Rook => 'r',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            _ => unreachable!("only valid promotion pieces are generated"),
        });
    }

    result
}

pub fn run() {
    let stdin = io::stdin();
    let mut board = Board::starting_position();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let fields: Vec<&str> = line.split_whitespace().collect();

        match fields.as_slice() {
            ["uci"] => {
                println!("id name honeycombee");
                println!("id author Leon Mamic, Zoe Posokhova");
                println!("uciok");
            }
            ["isready"] => println!("readyok"),
            ["ucinewgame"] => board = Board::starting_position(),

            ["position", "startpos", rest @ ..] => {
                board = Board::starting_position();

                if let Some(move_index) = rest.iter().position(|&part| part == "moves") {
                    for text in &rest[move_index + 1..] {
                        let Some(mv) = parse_move(&mut board, text) else {
                            eprintln!("invalid UCI move: {text}");
                            break;
                        };
                        board.make_move(mv);
                    }
                }
            }

            ["go", rest @ ..] => {
                let depth = rest
                    .windows(2)
                    .find(|pair| pair[0] == "depth")
                    .and_then(|pair| pair[1].parse().ok())
                    .unwrap_or(3);

                match find_best_move(&mut board, depth) {
                    Some(mv) => println!("bestmove {}", move_to_uci(mv)),
                    None => print!("bestmove 0000"),
                }
            }

            ["quit"] => break,
            _ => {}
        }

        io::stdout().flush().expect("stdout flush failed");
    }
}
use crate::{
    board::{Board, Color, Move, Piece},
    search::find_best_move,
};
use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

const MAX_SEARCH_DEPTH: u32 = 64;
const MOVE_OVERHEAD_MS: u64 = 200;

fn square(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();

    if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1])
    {
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

fn go_time_limit(board: &Board, fields: &[&str]) -> Duration {
    let values_after = |name: &str| {
        fields
            .windows(2)
            .find(|pair| pair[0] == name)
            .and_then(|pair| pair[1].parse::<u64>().ok())
    };

    let (remaining, increment) = match board.side_to_move {
        Color::White => (values_after("wtime"), values_after("winc")),
        Color::Black => (values_after("btime"), values_after("binc")),
    };

    let Some(remaining) = remaining else {
        return Duration::from_secs(1);
    };

    let increment = increment.unwrap_or(0);

    // spend 1/30 of our time and most of the increment
    let prefered = remaining / 30 + increment * 3 / 4;

    let maximum_safe = remaining.saturating_sub(MOVE_OVERHEAD_MS).max(1);
    let budget = prefered.clamp(1, maximum_safe);

    Duration::from_millis(budget)
}

pub fn run() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    run_with(stdin.lock(), &mut stdout);
}

fn run_with<R: BufRead, W: Write>(input: R, output: &mut W) {
    let mut board = Board::starting_position();

    for line in input.lines() {
        let Ok(line) = line else { break };
        let fields: Vec<&str> = line.split_whitespace().collect();

        match fields.as_slice() {
            ["uci"] => {
                writeln!(output, "id name honeycombee").expect("UCI output failed");
                writeln!(output, "id author Leon Mamic, Zoe Posokhova").expect("UCI output failed");
                writeln!(output, "uciok").expect("UCI output failed");
            }
            ["isready"] => writeln!(output, "readyok").expect("UCI output failed"),
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
                    .unwrap_or(MAX_SEARCH_DEPTH);

                let time_limit = go_time_limit(&board, rest);

                match find_best_move(&mut board, depth, time_limit) {
                    Some(mv) => writeln!(output, "bestmove {}", move_to_uci(mv)),
                    None => writeln!(output, "bestmove 0000"),
                }
                .expect("UCI output failed");
            }

            ["quit"] => break,
            _ => {}
        }

        output.flush().expect("UCI output flush failed");
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{parse_move, run_with};
    use crate::board::Board;

    #[test]
    fn uci_transcript_reports_ready_and_a_legal_best_move() {
        let input =
            Cursor::new("uci\nisready\nposition startpos moves e2e4 e7e5\ngo depth 1\nquit\n");
        let mut output = Vec::new();

        run_with(input, &mut output);

        let output = String::from_utf8(output).expect("UCI output must be UTF-8");
        assert!(output.contains("uciok\n"));
        assert!(output.contains("readyok\n"));

        let best_move = output
            .lines()
            .find_map(|line| line.strip_prefix("bestmove "))
            .expect("go must return a bestmove");
        let mut board = Board::starting_position();
        for text in ["e2e4", "e7e5"] {
            let mv = parse_move(&mut board, text).expect("setup move must be legal");
            board.make_move(mv);
        }

        assert!(parse_move(&mut board, best_move).is_some());
    }
}

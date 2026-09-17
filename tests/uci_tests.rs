use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn go_emits_parseable_statistics_before_bestmove() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_honeycomb"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"uci\nisready\nposition startpos\ngo depth 2\nquit\n")
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.ends_with('\n'));
    let lines: Vec<_> = output.lines().collect();
    assert!(lines.contains(&"uciok"));
    assert!(lines.contains(&"readyok"));
    let stats: Vec<_> = lines
        .iter()
        .filter(|line| line.starts_with("info depth "))
        .collect();
    assert_eq!(stats.len(), 2, "{output}");
    for (index, line) in stats.iter().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let number = |name: &str| -> u64 {
            fields.windows(2).find(|pair| pair[0] == name).unwrap()[1]
                .parse()
                .unwrap()
        };
        assert_eq!(number("depth"), index as u64 + 1);
        assert!(number("nodes") > 0);
        let _ = number("nps");
        let _ = number("time");
        assert!(line.contains(" score cp "));
    }
    let qnodes: Vec<u64> = lines
        .iter()
        .filter_map(|line| line.strip_prefix("info string qnodes "))
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(qnodes.len(), 2);
    assert!(qnodes[0] > 0 && qnodes[1] > qnodes[0]);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("bestmove "))
            .count(),
        1
    );
    assert!(lines.last().unwrap().starts_with("bestmove "));
}

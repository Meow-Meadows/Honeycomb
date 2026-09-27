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
        .write_all(b"uci\nsetoption name Hash value 1\nisready\nposition startpos\ngo depth 2\ngo depth 2\nquit\n")
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
    let max_hash_mb = if usize::BITS >= 64 { 6144 } else { 2047 };
    assert!(lines.contains(
        &format!("option name Hash type spin default 16 min 1 max {max_hash_mb}").as_str()
    ));
    let stats: Vec<_> = lines
        .iter()
        .filter(|line| line.starts_with("info depth "))
        .collect();
    assert_eq!(stats.len(), 4, "{output}");
    for (index, line) in stats.iter().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let number = |name: &str| -> u64 {
            fields.windows(2).find(|pair| pair[0] == name).unwrap()[1]
                .parse()
                .unwrap()
        };
        assert_eq!(number("depth"), (index % 2) as u64 + 1);
        assert!(number("nodes") > 0);
        let _ = number("nps");
        let _ = number("time");
        assert!(number("hashfull") <= 1000);
        assert!(!line.contains(" tbhits "));
        assert!(line.contains(" score cp "));
    }
    let qnodes: Vec<u64> = lines
        .iter()
        .filter_map(|line| line.strip_prefix("info string qnodes "))
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(qnodes.len(), 4);
    assert!(qnodes[0] > 0 && qnodes[1] > qnodes[0]);
    assert!(qnodes[2] > 0 && qnodes[3] > 0);
    let tt_hits: Vec<u64> = lines
        .iter()
        .filter_map(|line| line.strip_prefix("info string tthits "))
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(tt_hits.len(), 4);
    assert!(
        tt_hits[3] > 0,
        "second search should reuse the TT: {output}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("bestmove "))
            .count(),
        2
    );
    assert!(lines.last().unwrap().starts_with("bestmove "));
}

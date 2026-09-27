use crate::board::Move;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone)]
struct Entry {
    key: u64,
    depth: u8,
    score: i32,
    bound: Bound,
    best_move: Option<Move>,
}

struct TranspositionTable {
    entries: Vec<Option<Entry>>,
}

impl TranspositionTable {
    fn new(slot_count: usize) -> Self {
        Self {
            entries: vec![None; slot_count],
        }
    }

    fn index(&self, key: u64) -> usize {
        key as usize % self.entries.len()
    }

    fn get(&self, key: u64) -> Option<Entry> {
        self.entries[self.index(key)]
            .filter(|entry| entry.key == key)
    }

    fn store(&mut self, entry: Entry) {
        let index = self.index(entry.key);

        let replace = self.entries[index]
            .is_none_or(|old| entry.depth >= old.depth || old.key == entry.key);

        if replace {
            self.entries[index] = Some(entry);
        }
    }
}
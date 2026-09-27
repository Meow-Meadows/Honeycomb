use crate::board::Move;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone)]
pub(crate) struct Entry {
    pub(crate) key: u64,
    pub(crate) depth: u8,
    pub(crate) score: i32,
    pub(crate) bound: Bound,
    pub(crate) best_move: Option<Move>,
}

pub(crate) struct TranspositionTable {
    entries: Vec<Option<Entry>>,
}

impl TranspositionTable {
    pub(crate) fn new(slot_count: usize) -> Self {
        Self {
            entries: vec![None; slot_count.max(1)],
        }
    }

    fn index(&self, key: u64) -> usize {
        key as usize % self.entries.len()
    }

    pub(crate) fn get(&self, key: u64) -> Option<Entry> {
        self.entries[self.index(key)].filter(|entry| entry.key == key)
    }

    pub(crate) fn store(&mut self, entry: Entry) {
        let index = self.index(entry.key);

        let replace = self.entries[index].is_none_or(|old| entry.depth >= old.depth);

        if replace {
            self.entries[index] = Some(entry);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{Bound, Entry, TranspositionTable};
    use crate::board::{Move, Piece};

    fn entry(key: u64, depth: u8, score: i32) -> Entry {
        Entry {
            key,
            depth,
            score,
            bound: Bound::Exact,
            best_move: Some(Move {
                from: 12,
                to: 28,
                promotion: Some(Piece::Queen),
            }),
        }
    }

    #[test]
    fn retrieves_an_entry_for_the_same_key() {
        let mut table = TranspositionTable::new(4);
        table.store(entry(7, 3, 125));

        let found = table.get(7).expect("matching key should hit");
        assert_eq!(found.key, 7);
        assert_eq!(found.depth, 3);
        assert_eq!(found.score, 125);
        assert_eq!(found.bound, Bound::Exact);
        assert_eq!(found.best_move.unwrap().to, 28);
    }

    #[test]
    fn stores_each_bound_type() {
        let mut table = TranspositionTable::new(4);
        for (key, bound) in [(0, Bound::Exact), (1, Bound::Lower), (2, Bound::Upper)] {
            let mut item = entry(key, 1, 0);
            item.bound = bound;
            table.store(item);
            assert_eq!(table.get(key).unwrap().bound, bound);
        }
    }

    #[test]
    fn misses_for_an_empty_slot_or_different_key() {
        let mut table = TranspositionTable::new(4);
        assert!(table.get(1).is_none());

        table.store(entry(1, 3, 25));
        assert!(table.get(2).is_none());
    }

    #[test]
    fn full_key_check_rejects_a_slot_collision() {
        let mut table = TranspositionTable::new(4);
        table.store(entry(1, 3, 25));
        table.store(entry(5, 3, 40));

        assert!(table.get(1).is_none());
        assert_eq!(table.get(5).unwrap().score, 40);
    }

    #[test]
    fn replacement_keeps_deeper_colliding_entry() {
        let mut table = TranspositionTable::new(4);
        table.store(entry(1, 8, 80));
        table.store(entry(5, 7, 50));

        assert_eq!(table.get(1).unwrap().score, 80);
        assert!(table.get(5).is_none());
    }

    #[test]
    fn shallower_result_does_not_replace_deeper_result_for_same_key() {
        let mut table = TranspositionTable::new(4);
        table.store(entry(1, 8, 80));
        table.store(entry(1, 3, 30));

        let found = table.get(1).unwrap();
        assert_eq!(found.depth, 8);
        assert_eq!(found.score, 80);
    }

    #[test]
    fn deeper_entry_replaces_shallower_collision() {
        let mut table = TranspositionTable::new(4);
        table.store(entry(1, 3, 30));
        table.store(entry(5, 4, 50));

        assert!(table.get(1).is_none());
        assert_eq!(table.get(5).unwrap().score, 50);
    }

    #[test]
    fn zero_requested_slots_still_create_a_usable_one_slot_table() {
        let mut table = TranspositionTable::new(0);
        table.store(entry(1, 1, 10));
        assert_eq!(table.get(1).unwrap().score, 10);
    }
}

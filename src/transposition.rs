use crate::board::Move;
use std::mem::size_of;

pub(crate) const MIN_HASH_MB: usize = 1;
pub(crate) const MAX_HASH_MB: usize = if usize::BITS >= 64 { 6144 } else { 2047 };

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

    pub(crate) fn try_with_megabytes(megabytes: usize) -> Option<Self> {
        if !(MIN_HASH_MB..=MAX_HASH_MB).contains(&megabytes) {
            return None;
        }

        let bytes = megabytes.checked_mul(1024 * 1024)?;
        let slot_count = bytes / size_of::<Option<Entry>>();
        let mut entries = Vec::new();
        entries.try_reserve_exact(slot_count).ok()?;
        entries.resize(slot_count, None);

        Some(Self { entries })
    }

    pub(crate) fn clear(&mut self) {
        self.entries.fill(None);
    }

    pub(crate) fn hashfull(&self) -> u16 {
        let sample_size = self.entries.len().min(1000);
        if sample_size == 0 {
            return 0;
        }

        let occupied = self.entries[..sample_size]
            .iter()
            .filter(|entry| entry.is_some())
            .count();

        (occupied * 1000 / sample_size) as u16
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
    use super::{Bound, Entry, MAX_HASH_MB, TranspositionTable};
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
    fn megabyte_allocation_uses_entry_size_and_rejects_out_of_range_values() {
        let table = TranspositionTable::try_with_megabytes(1).unwrap();
        assert_eq!(
            table.entries.len() * size_of::<Option<Entry>>(),
            (1024 * 1024 / size_of::<Option<Entry>>()) * size_of::<Option<Entry>>()
        );
        assert!(TranspositionTable::try_with_megabytes(0).is_none());
        assert!(TranspositionTable::try_with_megabytes(MAX_HASH_MB + 1).is_none());
    }

    #[test]
    fn clear_resets_hashfull_and_entries() {
        let mut table = TranspositionTable::new(1000);
        table.store(entry(1, 1, 10));
        assert_eq!(table.hashfull(), 1);

        table.clear();
        assert_eq!(table.hashfull(), 0);
        assert!(table.get(1).is_none());
    }

    #[test]
    fn zero_requested_slots_still_create_a_usable_one_slot_table() {
        let mut table = TranspositionTable::new(0);
        table.store(entry(1, 1, 10));
        assert_eq!(table.get(1).unwrap().score, 10);
    }
}

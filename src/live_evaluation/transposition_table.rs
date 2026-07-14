use crate::board::bitmap::{BITMAP_COLUMN_HEIGHT, BITMAP_ROW_WIDTH};
use crate::board::key_methods::PositionKey;
use crate::live_evaluation::mtd::{MAXIMUM_SCORE, NEGATED_MINIMUM_SCORE};

// Must be called with n > 2
const fn is_prime(n: u32) -> bool {
    let mut divisor = 2;
    while divisor * divisor <= n {
        if n % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
}

// The Table Size should be a prime number to ensure a uniform usage of the table
const MINIMUM_LOG_OF_TABLE_ENTRIES: u8 = 23;
const TABLE_SIZE_U32: u32 = {
    let mut table_size = 1 << MINIMUM_LOG_OF_TABLE_ENTRIES;
    while !is_prime(table_size) {
        table_size += 1;
    }
    table_size
};

// Needed because this file casts u32 to usize
const _: () = {
    // size_of returns bytes. 4 bytes = 32 bits.
    if core::mem::size_of::<usize>() < 4 {
        panic!("Compilation failed: This project requires a 32-bit or 64-bit target architecture.");
    }
};
const TABLE_SIZE: usize = TABLE_SIZE_U32 as usize;


// Being prime also allows the Chinese Remainder Theorem to apply
// It ensures that the there are no collisions between two full keys where:
// full_key_a != full_key_b
// truncated_key_a == truncated_key_b
// table_index_a == table_index_b
type TruncatedKey = u32;
const _: () = {
    let full_key_bits = BITMAP_ROW_WIDTH * BITMAP_COLUMN_HEIGHT;
    assert!(
        MINIMUM_LOG_OF_TABLE_ENTRIES + 32 >= full_key_bits,
        "There is a possibility of collisions in hash table"
    );
};


pub type TableValue = u8;

const STORED_UPPER_BOUND_OFFSET: u8 = 1 + NEGATED_MINIMUM_SCORE;
const MAX_STORED_UPPER_BOUND: u8 = MAXIMUM_SCORE + STORED_UPPER_BOUND_OFFSET;
const STORED_LOWER_BOUND_OFFSET: u8 = MAX_STORED_UPPER_BOUND + 1 + NEGATED_MINIMUM_SCORE;
const MAX_STORED_LOWER_BOUND: u8 = MAXIMUM_SCORE + STORED_LOWER_BOUND_OFFSET;


#[derive(Clone, Copy)]
pub struct TableKey {
    table_index: u32,
    truncated_key: u32,
}

impl From<PositionKey> for TableKey {
    fn from(position_key: PositionKey) -> Self {
        Self {
            // This is safe casting
            // IMO, a u64 modulo u32 should just return a u32
            // Because modulo operations can never return anything greater than the modulus
            table_index: (position_key % TABLE_SIZE_U32 as u64) as u32,
            // This cast does intentional truncation
            truncated_key: position_key as TruncatedKey,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TableResult {
    Miss,
    LowerBound(i8),
    UpperBound(i8),
}

pub struct TranspositionTable {
    truncated_keys: Vec<TruncatedKey>,
    saved_values: Vec<TableValue>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            truncated_keys: vec![0; TABLE_SIZE],
            saved_values: vec![0; TABLE_SIZE],
        }
    }

    pub fn put_upper_bound(&mut self, table_key: TableKey, upper_bound: i8) {
        let saved_value = (upper_bound + STORED_UPPER_BOUND_OFFSET as i8) as u8;
        self.truncated_keys[table_key.table_index as usize] = table_key.truncated_key;
        self.saved_values[table_key.table_index as usize] = saved_value;
    }

    pub fn put_lower_bound(&mut self, table_key: TableKey, lower_bound: i8) {
        let saved_value = (lower_bound + STORED_LOWER_BOUND_OFFSET as i8) as u8;
        self.truncated_keys[table_key.table_index as usize] = table_key.truncated_key;
        self.saved_values[table_key.table_index as usize] = saved_value;
    }

    pub fn get(&self, table_key: TableKey) -> TableResult {
        let saved_truncated_key = self.truncated_keys[table_key.table_index as usize];
        if saved_truncated_key != table_key.truncated_key {
            return TableResult::Miss;
        }
        let saved_value = self.saved_values[table_key.table_index as usize];
        match saved_value {
            0 => TableResult::Miss,
            1..=MAX_STORED_UPPER_BOUND => TableResult::UpperBound(
                saved_value as i8 - STORED_UPPER_BOUND_OFFSET as i8
            ),
            _ => {
                debug_assert!(saved_value <= MAX_STORED_LOWER_BOUND);
                TableResult::LowerBound(
                    saved_value as i8 - STORED_LOWER_BOUND_OFFSET as i8
                )
            }
        }
    }

    pub fn reset(&mut self) {
        self.truncated_keys.fill(0);
        self.saved_values.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
    fn test_table_is_initially_empty() {
        let table = TranspositionTable::new();
        let dummy_table_key = TableKey::from(0);
        assert_eq!(TableResult::Miss, table.get(dummy_table_key));
    }

	#[test]
    fn test_put_and_get_lower_bound() {
        let mut table = TranspositionTable::new();
        let dummy_table_key = TableKey::from(0);
        let dummy_bound = 1;

        table.put_lower_bound(dummy_table_key, dummy_bound);
        assert_eq!(TableResult::LowerBound(dummy_bound), table.get(dummy_table_key));
    }

	#[test]
    fn test_put_and_get_upper_bound() {
        let mut table = TranspositionTable::new();
        let dummy_table_key = TableKey::from(0);
        let dummy_bound = 1;

        table.put_upper_bound(dummy_table_key, dummy_bound);
        assert_eq!(TableResult::UpperBound(dummy_bound), table.get(dummy_table_key));
    }

	#[test]
    fn test_collision_picks_later() {
        let mut table = TranspositionTable::new();
        let dummy_table_key = TableKey::from(0);
        let dummy_bound = 1;

        table.put_lower_bound(dummy_table_key, dummy_bound);
        table.put_upper_bound(dummy_table_key, dummy_bound);
        assert_eq!(TableResult::UpperBound(dummy_bound), table.get(dummy_table_key));

        table.put_lower_bound(dummy_table_key, dummy_bound);
        assert_eq!(TableResult::LowerBound(dummy_bound), table.get(dummy_table_key));
    }

	#[test]
    fn test_reset_causes_miss() {
        let mut table = TranspositionTable::new();
        let dummy_table_key = TableKey::from(0);
        let dummy_bound = 1;

        table.put_lower_bound(dummy_table_key, dummy_bound);
        assert_eq!(TableResult::LowerBound(dummy_bound), table.get(dummy_table_key));

        table.reset();
        assert_eq!(TableResult::Miss, table.get(dummy_table_key));
    }
}

use std::{mem::MaybeUninit, usize};

use crate::board::{Position, GAME_WIDTH};

// Odd number example [3, 2, 4, 1, 5, 0, 6]
// Even number example [4, 3, 5, 2, 6, 1, 7, 0]
pub(crate) const INSIDE_OUT_COLUMN_ORDER: [u8; GAME_WIDTH as usize] = {
    let mut default_column_order = [0; GAME_WIDTH as usize];
    let middle_column = GAME_WIDTH / 2;
    let mut offset_from_middle = 0;

    let mut i = 0;
    while i < GAME_WIDTH {
        if i % 2 == 1 { offset_from_middle += 1; }

        let column = if i % 2 == 0 {
            middle_column + offset_from_middle
        } else {
            middle_column - offset_from_middle
        };
        default_column_order[i as usize] = column;

        i += 1;
    }
    default_column_order
};

// Odd number example [6, 0, 5, 1, 4, 2, 3]
// Even number example [0, 7, 1, 6, 2, 5, 3, 4]

// Meant to be iterated over when adding to the SortedChildPositionIterator
// Iterate over this, because SortedChildPositionIterator behaves like a stack in the case of ties
// Therefore, if everything is tied, the column order will be INSIDE_OUT_COLUMN_ORDER
pub(crate) const OUTSIDE_IN_COLUMN_ORDER: [u8; GAME_WIDTH as usize] = {
    let mut reversed_default_column_order = [0; GAME_WIDTH as usize];
    let mut i = 0;
    while i < GAME_WIDTH as usize {
        reversed_default_column_order[GAME_WIDTH as usize - i - 1] = INSIDE_OUT_COLUMN_ORDER[i];
        i += 1;
    }
    reversed_default_column_order
};

#[derive(Clone, Copy)]
struct ScoredEntry {
    pub child_position: Position,
    pub score: u8,
}

pub(crate) struct ScoredPositionIterator {
    entries: [MaybeUninit<ScoredEntry>; GAME_WIDTH as usize],
    size: u8,
}

impl ScoredPositionIterator {
    pub(crate) fn new() -> Self {
        Self { entries: [MaybeUninit::uninit(); GAME_WIDTH as usize], size: 0 }
    }

    pub(crate) fn add(&mut self, child_position: Position, score: u8) {
        // The index we are looking at (initially, the first empty index)
        let mut vacant_index = self.size as usize;

        // As long as the previous entry is strictly bigger, shift that right once
        while vacant_index != 0 && unsafe { self.entries[vacant_index - 1].assume_init().score } > score {
            self.entries[vacant_index] = self.entries[vacant_index - 1];
            vacant_index -= 1;
        }

        // Place our entry at the now empty spot
        self.entries[vacant_index] = MaybeUninit::new(ScoredEntry { child_position, score});
        self.size += 1;
    }
}

impl Iterator for ScoredPositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            Some(unsafe { self.entries[self.size as usize].assume_init() }.child_position)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::board::testing_constants::BLANK_POSITION;

    use super::*;

    fn get_single_square_positions() -> Vec<Position> {
        (0..GAME_WIDTH).map(|col| {
            let mut single_square_position = BLANK_POSITION;
            single_square_position.play_column(col);
            single_square_position
        }).collect()
    }

    // Tests that the iterator returns results then repeatedly returns None
    #[test]
    fn test_sorter_with_zero_elements() {
        let mut scored_position_iterator = ScoredPositionIterator::new();
        assert_eq!(None, scored_position_iterator.next());
        assert_eq!(None, scored_position_iterator.next());
    }

    #[test]
    fn test_sorter_with_one_elements() {
        let input_positions = get_single_square_positions();
        let mut scored_position_iterator = ScoredPositionIterator::new();

        scored_position_iterator.add(input_positions[0], 0);

        assert_eq!(Some(input_positions[0]), scored_position_iterator.next());
        assert_eq!(None, scored_position_iterator.next());
        assert_eq!(None, scored_position_iterator.next());
    }

    #[test]
    fn test_sorter_with_two_elements() {
        let input_positions = get_single_square_positions();
        let mut scored_position_iterator = ScoredPositionIterator::new();

        scored_position_iterator.add(input_positions[0], 0);
        scored_position_iterator.add(input_positions[1], 0);

        assert_eq!(Some(input_positions[1]), scored_position_iterator.next());
        assert_eq!(Some(input_positions[0]), scored_position_iterator.next());
        assert_eq!(None, scored_position_iterator.next());
        assert_eq!(None, scored_position_iterator.next());
    }

    // Tests the fact that if we put in all ties, it will function as a stack
    #[test]
    fn test_stable_sort() {
        let input_positions = get_single_square_positions();
        let mut scored_position_iterator = ScoredPositionIterator::new();

        for &position in &input_positions {
            scored_position_iterator.add(position, 0);
        }

        let mut iterator_result: Vec<Position> = scored_position_iterator.collect();
        iterator_result.reverse();
        assert_eq!(input_positions, iterator_result)
    }

    #[test]
    fn test_normal_sort() {
        let input_positions = get_single_square_positions();
        let mut scored_position_iterator = ScoredPositionIterator::new();

        let scores = [3, 2, 6, 7, 5, 4, 1];
        let expected_order = [3, 2, 4, 5, 0, 1, 6];

        let mut expected_output_positions = Vec::new();

        for i in 0..scores.len() {
            expected_output_positions.push(input_positions[expected_order[i]]);
            scored_position_iterator.add(input_positions[i], scores[i]);
        }

        let actual_positions: Vec<Position> = scored_position_iterator.collect();
        assert_eq!(expected_output_positions, actual_positions);
    }

    // Some ties, some different scores
    // Expect to sort as much as possible, then treat the ties like a stack
    #[test]
    fn test_hybrid_sort() {
        let input_positions = get_single_square_positions();
        let mut scored_position_iterator = ScoredPositionIterator::new();

        let scores = [3, 3, 6, 7, 5, 5, 1];
        let expected_order = [3, 2, 5, 4, 1, 0, 6];

        let mut expected_output_positions = Vec::new();

        for i in 0..scores.len() {
            expected_output_positions.push(input_positions[expected_order[i]]);
            scored_position_iterator.add(input_positions[i], scores[i]);
        }

        let actual_positions: Vec<Position> = scored_position_iterator.collect();
        assert_eq!(expected_output_positions, actual_positions);
    }
}

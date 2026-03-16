use crate::board::{bitmap, Position};
use crate::board::bitmap::{Bitmap, BITMAP_COLUMN_HEIGHT, BITMAP_ROW_WIDTH, BOTTOM_ROW_MASK, PLAYABLE_BOARD_MASK};

pub type PositionKey = u64;

#[derive(Clone, Copy)]
struct DownShifter {
    shift_amount: u8,
    shift_mask: Bitmap,
}

const DOWN_SHIFTS_OPERATIONS_NEEDED: usize = {
    ((BITMAP_COLUMN_HEIGHT - 1).ilog2() + 1) as usize
};
const DOWN_SHIFTERS: [DownShifter; DOWN_SHIFTS_OPERATIONS_NEEDED] = {
    /* Basic idea, shift the key down 1 (then AND with yourself), then 2, then 4, etc.

         1 1 0 0 0 0 0       1 1 0 0 0 0 0       1 1 0 0 0 0 0       1 1 0 0 0 0 0
         0 0 1 1 0 0 0       1 1 1 1 0 0 0       1 1 1 1 0 0 0       1 1 1 1 0 0 0
         0 0 0 0 1 1 0       0 0 1 1 1 1 0       1 1 1 1 1 1 0       1 1 1 1 1 1 0
         0 0 0 0 0 0 0  ->   0 0 0 0 1 1 0  ->   1 1 1 1 1 1 0  ->   1 1 1 1 1 1 0
         0 0 0 0 0 0 0       0 0 0 0 0 0 0       0 0 1 1 1 1 0       1 1 1 1 1 1 0
         0 0 0 0 0 0 0       0 0 0 0 0 0 0       0 0 0 0 1 1 0       1 1 1 1 1 1 0
         0 0 0 0 0 0 1       0 0 0 0 0 0 1       0 0 0 0 0 0 1       1 1 1 1 1 1 1
    
    The problem is that if column 2 has a '1' at the bottom,
    then shifting right has the effect of placing this '1' at the top of column 1
    To do this, we need to "noUnderflow" bitmaps, which specify where '1's should
    even be possible after performing a shift operation.
    
           One Shift           Two Shifts         Four Shifts
         0 0 0 0 0 0 0       0 0 0 0 0 0 0       0 0 0 0 0 0 0
         1 1 1 1 1 1 1       0 0 0 0 0 0 0       0 0 0 0 0 0 0
         1 1 1 1 1 1 1       1 1 1 1 1 1 1       0 0 0 0 0 0 0
         1 1 1 1 1 1 1  ->   1 1 1 1 1 1 1  ->   0 0 0 0 0 0 0
         1 1 1 1 1 1 1       1 1 1 1 1 1 1       1 1 1 1 1 1 1
         1 1 1 1 1 1 1       1 1 1 1 1 1 1       1 1 1 1 1 1 1
         1 1 1 1 1 1 1       1 1 1 1 1 1 1       1 1 1 1 1 1 1 */
    let mut down_shifters = [DownShifter{shift_amount: 0, shift_mask: 0}; DOWN_SHIFTS_OPERATIONS_NEEDED];

    let mut index = 0;
    let mut shift_amount = 1;

    while index < DOWN_SHIFTS_OPERATIONS_NEEDED {
        let shift_column_mask = (1 << BITMAP_COLUMN_HEIGHT - shift_amount) - 1;
        let shift_mask = BOTTOM_ROW_MASK * shift_column_mask;

        down_shifters[index] = DownShifter{ shift_amount, shift_mask };
        index += 1;
        shift_amount <<= 1;
    }
    down_shifters
};

const PAST_MIDDLE_COLUMN_INDEX: u8 = (BITMAP_ROW_WIDTH + 1) / 2;


impl Position {
    /// Turn the position into a unique key
    pub fn get_key(&self) -> PositionKey {
		/* Original definition of the key was position + mask + BOTTOM_MASK
		This has a conceptual interpretation and is guaranteed to be unique per position
		But because this is unique, so is just position + mask */
        self.current_player_stones + self.both_players_mask
    }

    /// "Mirror" means that the columns are flipped about the middle
    pub fn get_mirror_key(&self) -> PositionKey {
        let key = self.get_key();

        let mut mirror_key = 0;

        // Up to and including the middle column, shift left to reach the mirror column
        for i in 0..PAST_MIDDLE_COLUMN_INDEX {
            mirror_key += (key & bitmap::compute_bitmap_column_mask(i)) << BITMAP_COLUMN_HEIGHT * (BITMAP_ROW_WIDTH - 1 - 2 * i);
        }

        // Past the middle column, shift right right to reach mirror column
        for i in PAST_MIDDLE_COLUMN_INDEX..BITMAP_ROW_WIDTH {
            mirror_key += (key & bitmap::compute_bitmap_column_mask(i)) >> BITMAP_COLUMN_HEIGHT * (2 * i - BITMAP_ROW_WIDTH + 1);
        }

        mirror_key
    }

    /// Construct a position from a unique key
    pub fn from_key(key: PositionKey) -> Self {
        // Build the key back to how it was originally defined
        let original_key = key + BOTTOM_ROW_MASK;
        let mut both_players_mask = original_key;
        for down_shifter in DOWN_SHIFTERS {
            both_players_mask |= both_players_mask >> down_shifter.shift_amount & down_shifter.shift_mask
        }

        // Finally, shift everything down 1 because the key puts each column's '1'
        // Just above where the mask would have it
        both_players_mask = both_players_mask >> 1 & PLAYABLE_BOARD_MASK;
        let current_player_stones = original_key & both_players_mask;
        let moves_played = bitmap::count_ones_in_bitmap(both_players_mask);
        Position { current_player_stones, both_players_mask, moves_played }
    }

    /// Construct a position from a unique key and the number of moves played
    /// The number of moves played can be concluded from the unique key
    /// However, if both are already known, then calling this method is slightly faster
    pub fn from_key_and_moves_played(key: PositionKey, moves_played: u8) -> Self {
        // Build the key back to how it was originally defined
        let original_key = key + BOTTOM_ROW_MASK;
        let mut both_players_mask = original_key;
        for down_shifter in DOWN_SHIFTERS {
            both_players_mask |= both_players_mask >> down_shifter.shift_amount & down_shifter.shift_mask
        }

        // Finally, shift everything down 1 because the key puts each column's '1'
        // Just above where the mask would have it
        both_players_mask = both_players_mask >> 1 & PLAYABLE_BOARD_MASK;
        let current_player_stones = original_key & both_players_mask;
        Position { current_player_stones, both_players_mask, moves_played }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{testing_constants::{BLANK_POSITION, COMPLEX_POSITION, TESTING_POSITIONS}, Position, GAME_WIDTH};

    #[test]
    fn test_get_key() {
        assert_eq!(0, BLANK_POSITION.get_key());
        assert_eq!(0x10A67125D88, COMPLEX_POSITION.get_key());
    }

    #[test]
    fn test_get_mirror_key() {
        /* The mirrored complex position should look like:
                * * * * 0 * *
                * 0 0 1 0 1 *
                * 0 0 1 1 1 *
                * 0 1 0 0 1 0
                * 1 1 0 1 0 0
                * 0 1 1 0 0 1
         */
        let mirror_complex = Position{current_player_stones: 0x4E0A321C100, both_players_mask: 0x1CFBF3E7CF80, moves_played: 29 };

        // Proves that the positions are different (non symmetrical position)
        assert_ne!(COMPLEX_POSITION.get_key(), mirror_complex.get_key());

        // But they are actually mirrors of each other
        assert_eq!(mirror_complex.get_key(), COMPLEX_POSITION.get_mirror_key());
        assert_eq!(COMPLEX_POSITION.get_key(), mirror_complex.get_mirror_key());

        let mut symmetrical_position = BLANK_POSITION;

        // Fill in the bottom-row
        for i in 0..GAME_WIDTH {
            symmetrical_position.play_column(i);
        }
        assert_eq!(symmetrical_position.get_key(), symmetrical_position.get_mirror_key());

        // Fill in the middle column
        symmetrical_position.play_column(GAME_WIDTH / 2);
        assert_eq!(symmetrical_position.get_key(), symmetrical_position.get_mirror_key());
    }

    #[test]
    fn test_from_key_round_trip() {
        let original = TESTING_POSITIONS;
        let after_round_trip = std::array::from_fn(|i| Position::from_key(TESTING_POSITIONS[i].get_key()));
        assert_eq!(original, after_round_trip);
    }

    #[test]
    fn test_from_key_and_moves_played_round_trip() {
        let original = TESTING_POSITIONS;
        let after_round_trip = std::array::from_fn(|i| {
            let key = TESTING_POSITIONS[i].get_key();
            let moves_played = TESTING_POSITIONS[i].moves_played;
            Position::from_key_and_moves_played(key, moves_played)
        });
        assert_eq!(original, after_round_trip);
    }
}


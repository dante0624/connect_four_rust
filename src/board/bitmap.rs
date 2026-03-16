use crate::board::{GAME_HEIGHT, GAME_WIDTH};

pub type Bitmap = u64;

pub(crate) const BITMAP_ROW_WIDTH: u8 = GAME_WIDTH;
pub(crate) const BITMAP_COLUMN_HEIGHT: u8 = GAME_HEIGHT + 1;

pub(crate) const UP_LEFT_DOWN_RIGHT_OFFSET: u8 = BITMAP_COLUMN_HEIGHT - 1;
pub(crate) const DOWN_LEFT_UP_RIGHT_OFFSET: u8 = BITMAP_COLUMN_HEIGHT + 1;

pub(crate) const BOTTOM_ROW_MASK: Bitmap = {
    let mut bottom_row = 0;
    let mut column = 0;
    while column < BITMAP_ROW_WIDTH {
        bottom_row += 1 << (column * BITMAP_COLUMN_HEIGHT);
        column += 1;
    }
    bottom_row
};
pub(crate) const PLAYABLE_BOARD_MASK: Bitmap = BOTTOM_ROW_MASK * ((1 << GAME_HEIGHT) - 1);


pub(crate) fn compute_bottom_of_column_mask(col: u8) -> Bitmap {
    1 << col * BITMAP_COLUMN_HEIGHT
}

pub(crate) fn compute_top_of_playable_column_mask(col: u8) -> Bitmap {
    1 << GAME_HEIGHT - 1 << col * BITMAP_COLUMN_HEIGHT
}

pub(crate) fn compute_playable_column_mask(col: u8) -> Bitmap {
    (1 << GAME_HEIGHT) - 1 << col * BITMAP_COLUMN_HEIGHT
}

pub(crate) fn compute_bitmap_column_mask(col: u8) -> Bitmap {
    (1 << BITMAP_COLUMN_HEIGHT) - 1 << col * BITMAP_COLUMN_HEIGHT
}

pub(crate) fn count_ones_in_bitmap(mut bitmap: Bitmap) -> u8 {
    let mut count = 0;
    while bitmap != 0 {
        bitmap &= bitmap - 1;
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Move this one over to transposition table UTs
    #[test]
    fn test_transposition_table_can_hold_bitmap() {
        // This needs to be true for the Chinese Remainder Theorem to hold in the transposition table
        // We really only store 31 bits of the key
        // We index based on numEntries, which is roughly 2^23
        // So to make the Chinese Remainder Theorem hold, the true key size must be <= (31 + 23 = 54 bits)
        assert!(BITMAP_ROW_WIDTH * BITMAP_COLUMN_HEIGHT <= 54);
    }

    #[test]
    fn test_compute_playable_column_mask() {
        assert_eq!(0x3F, compute_playable_column_mask(0));
        assert_eq!(0x7E00000, compute_playable_column_mask(3));
    }
}


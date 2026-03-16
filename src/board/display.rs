use std::fmt;

use crate::board::{bitmap, Position, GAME_HEIGHT, GAME_WIDTH};
use crate::board::bitmap::{Bitmap, BITMAP_COLUMN_HEIGHT, BITMAP_ROW_WIDTH, DOWN_LEFT_UP_RIGHT_OFFSET, UP_LEFT_DOWN_RIGHT_OFFSET};

fn stringify_bitmap(bitmap: Bitmap) -> String {
    let mut output_string = String::new();

    for from_top_row_index in 0..BITMAP_COLUMN_HEIGHT {
        let from_bottom_row_index = BITMAP_COLUMN_HEIGHT - from_top_row_index - 1;
        for column_index in 0..BITMAP_ROW_WIDTH {
            let char = if bitmap & 1 << column_index * BITMAP_COLUMN_HEIGHT + from_bottom_row_index == 0 {
                '0'
            } else {
                '1'
            };
            output_string.push(char);

            if column_index < BITMAP_ROW_WIDTH - 1 {
                output_string.push(' ');
            }
        }

        if from_top_row_index < BITMAP_COLUMN_HEIGHT - 1 {
            output_string.push('\n');
        }
    }

    output_string
}

fn stringify_position_board(position: &Position) -> String {
    let mut output_string = String::new();

    for from_top_row_index in 0..GAME_HEIGHT {
        let from_bottom_row_index = GAME_HEIGHT - from_top_row_index - 1;
        for column_index in 0..GAME_WIDTH {
            let bit = 1 << column_index * BITMAP_COLUMN_HEIGHT + from_bottom_row_index;
            let char = if position.both_players_mask & bit == 0 {
                '*'
            } else if position.current_player_stones & bit != 0 {
                '1'
            } else {
                '0'
            };
            output_string.push(char);

            if column_index < GAME_WIDTH - 1 {
                output_string.push(' ');
            }
        }

        if from_top_row_index < GAME_HEIGHT - 1 {
            output_string.push('\n');
        }
    }

    output_string
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "position:\n{}\n\nmoves_played: {}", stringify_position_board(self), self.moves_played)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "current_player_stones:\n{}\n\nboth_players_mask:\n{}\n\nkey: {:#X}\n\nposition:\n{}\n\nmoves_played: {}",
            stringify_bitmap(self.current_player_stones),
            stringify_bitmap(self.both_players_mask),
            self.get_key(),
            stringify_position_board(self),
            self.moves_played
        )
    }
}

impl Position {
    fn to_connect_four_indices(current_player_stones: Bitmap) -> Vec<u8> {
		let mut connect_four_bitmap = 0; // Bitmap with ones everywhere that connect 4 or more has happened

        // Horizontal
        let mut adjacent = current_player_stones & current_player_stones >> BITMAP_COLUMN_HEIGHT; // Indicates there is a chip here and 1 to the right
		adjacent &= adjacent >> 2 * BITMAP_COLUMN_HEIGHT; // Indicates there is a chip here and all 3 right squares
		let mut highlight = adjacent | adjacent << BITMAP_COLUMN_HEIGHT; // Start to highlight where these connect 4's are
		highlight |= highlight << 2 * BITMAP_COLUMN_HEIGHT;
		connect_four_bitmap |= highlight;

        // Diagonal 1, up-left to down-right
        adjacent = current_player_stones & current_player_stones >> UP_LEFT_DOWN_RIGHT_OFFSET;
        adjacent &= adjacent >> 2 * UP_LEFT_DOWN_RIGHT_OFFSET;
		highlight = adjacent | adjacent << UP_LEFT_DOWN_RIGHT_OFFSET;
		highlight |= highlight << 2 * UP_LEFT_DOWN_RIGHT_OFFSET;
		connect_four_bitmap |= highlight;

        // Diagonal 2, down-left to up-right
        adjacent = current_player_stones & current_player_stones >> DOWN_LEFT_UP_RIGHT_OFFSET;
        adjacent &= adjacent >> 2 * DOWN_LEFT_UP_RIGHT_OFFSET;
		highlight = adjacent | adjacent << DOWN_LEFT_UP_RIGHT_OFFSET;
		highlight |= highlight << 2 * DOWN_LEFT_UP_RIGHT_OFFSET;
		connect_four_bitmap |= highlight;

        // Vertical
        adjacent = current_player_stones & current_player_stones >> 1;
        adjacent &= adjacent >> 2;
		highlight = adjacent | adjacent << 1;
		highlight |= highlight << 2;
		connect_four_bitmap |= highlight;

		// Convert to int[], where int is the square index
        let mut square_indices = Vec::with_capacity(bitmap::count_ones_in_bitmap(connect_four_bitmap) as usize);
		let mut sliding_mask = 1;

        for column in 0..GAME_WIDTH {
            for row in 0..GAME_HEIGHT {
                if sliding_mask & connect_four_bitmap != 0 {
                    square_indices.push((GAME_HEIGHT - row - 1) * GAME_WIDTH + column);
                }
				sliding_mask <<= 1;
            }
			sliding_mask <<= 1; // There is an empty bit at the top we always need to ignore
        }

		square_indices
    }
}

#[cfg(test)]
mod tests {
    use crate::board::testing_constants::{CONNECT_TWENTY, DRAWN, VERTICAL, BLANK_POSITION, COMPLEX_POSITION};

    use super::*;

    #[test]
    fn test_display_blank() {
        let expected = "\
position:
* * * * * * *
* * * * * * *
* * * * * * *
* * * * * * *
* * * * * * *
* * * * * * *

moves_played: 0";
        assert_eq!(expected, format!("{}", BLANK_POSITION));
    }

    #[test]
    fn test_display_vertical() {
        let expected = "\
position:
* * * * * * *
* * * * * * *
* * * * * * *
1 0 * * * * *
1 0 * * * * *
1 0 * * * * *

moves_played: 6";
        assert_eq!(expected, format!("{}", VERTICAL));
    }

    #[test]
    fn test_display_complex() {
        let expected = "\
position:
* * 0 * * * *
* 1 0 1 0 0 *
* 1 1 1 0 0 *
0 1 0 0 1 0 *
0 0 1 0 1 1 *
1 0 0 1 1 0 *

moves_played: 29";
        assert_eq!(expected, format!("{}", COMPLEX_POSITION));
    }

    #[test]
    fn test_to_connect_four_indices() {
        assert!(Position::to_connect_four_indices(DRAWN.current_player_stones ^ DRAWN.both_players_mask).is_empty());

        let expected_twenty_indices = vec![
                 1,              5,
                     9,     11,
            14, 15, 16, 17, 18, 19, 20,
                    23, 24, 25,
                29,     31,     33,
            35,         38,         41,
        ];
        let mut actual_twenty_indices = Position::to_connect_four_indices(CONNECT_TWENTY.current_player_stones ^ CONNECT_TWENTY.both_players_mask);
        actual_twenty_indices.sort();
        assert_eq!(expected_twenty_indices, actual_twenty_indices);
    }
}

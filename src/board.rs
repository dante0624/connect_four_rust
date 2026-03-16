pub use bitmap::Bitmap;

pub mod bitmap;
pub mod display;
pub mod key_methods;
pub mod min_max_methods;

pub const GAME_WIDTH: u8 = 7;
pub const GAME_HEIGHT: u8 = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Position {
    current_player_stones: Bitmap,
    both_players_mask: Bitmap,
    pub moves_played: u8,
}

#[cfg(test)]
mod testing_constants {
    use super::*;

    /* Blank Position:
        * * * * * * *
        * * * * * * *
        * * * * * * *
        * * * * * * *
        * * * * * * *
        * * * * * * *
     */
    pub(crate) const BLANK_POSITION: Position = Position::new_blank_game();

    /* Vertical Position:
        * * * * * * *
        * * * * * * *
        * * * * * * *
        1 0 * * * * *
        1 0 * * * * *
        1 0 * * * * *
     */
    pub(crate) const VERTICAL: Position = Position { current_player_stones: 0x7, both_players_mask: 0x387, moves_played: 6 };

    /* Horizontal Position:
        * * * * * * *
        * * * * * * *
        * * * * * * *
        * * * * * * *
        * * 0 0 0 * *
        * * 1 1 1 * *
     */
    pub(crate) const HORIZONTAL: Position = Position {current_player_stones: 0x10204000, both_players_mask: 0x3060C000, moves_played: 6 };

    /* Up-Left-Down-Right Diagonal Position:
        * * * * * * *
        * * * * * * *
        * 1 1 * * * *
        * 0 0 1 * * *
        * 1 1 0 1 * *
        0 0 0 1 0 * *
     */
    pub(crate) const UP_LEFT_DOWN_RIGHT_DIAGONAL: Position = Position { current_player_stones: 0x20A28500, both_players_mask: 0x30E3C781, moves_played: 14 };

    /* Down-Left-Up-Right Diagonal Position:
        * * * * * * *
        * * * * * * *
        * * * * 1 1 *
        * * * 1 0 0 *
        * * 1 0 1 1 *
        * * 0 1 0 0 0
     */
    pub(crate) const DOWN_LEFT_UP_RIGHT_DIAGONAL: Position = Position { current_player_stones: 0x50A0A08000, both_players_mask: 0x478F0E0C000, moves_played: 14 };

	/* Drawn Position:
	   0 0 0 1 0 0 1
	   0 1 1 0 1 1 1
	   0 0 0 1 0 0 1
	   1 1 1 0 1 1 0
	   1 0 0 1 0 0 0
	   1 1 1 0 1 1 0
	*/
	pub(crate) const DRAWN: Position = Position { current_player_stones: 0xD0A955454A87, both_players_mask: 0xFDFBF7EFDFBF, moves_played: 42 };

	/* Connect 20 (just won in middle column, the numbers switch. So the 0's won):
	   1 0 1 * 1 0 *
	   1 1 0 * 0 1 1
	   0 0 0 0 0 0 0
	   1 1 0 0 0 1 1
	   1 0 1 0 1 0 1
	   0 1 1 0 1 1 0
	*/
	pub(crate) const CONNECT_TWENTY: Position = Position { current_player_stones: 0x58AA3008CAB6, both_players_mask: 0x7DFBF1EFDFBF, moves_played: 39 };

    /* Complex Position:
		* * 0 * * * *
		* 1 0 1 0 0 *
		* 1 1 1 0 0 *
		0 1 0 0 1 0 *
		0 0 1 0 1 1 *
		1 0 0 1 1 0 *
     */
    pub(crate) const COMPLEX_POSITION: Position = Position { current_player_stones: 0x1073228E01, both_players_mask: 0xF9F3EFCF87, moves_played: 29 };

    pub(crate) const TESTING_POSITIONS: [Position; 8] = [
        BLANK_POSITION,
        VERTICAL,
        HORIZONTAL,
        UP_LEFT_DOWN_RIGHT_DIAGONAL,
        DOWN_LEFT_UP_RIGHT_DIAGONAL,
        DRAWN,
        CONNECT_TWENTY,
        COMPLEX_POSITION,
    ];
}


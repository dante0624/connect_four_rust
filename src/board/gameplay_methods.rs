use std::error::Error;
use std::fmt::Display;

use thiserror::Error;

use crate::board::bitmap::{BITMAP_COLUMN_HEIGHT, DOWN_LEFT_UP_RIGHT_OFFSET, UP_LEFT_DOWN_RIGHT_OFFSET};
use crate::board::{bitmap, Bitmap, Position, GAME_HEIGHT, GAME_WIDTH};

/* TODO: Expose an interface where you can play a column without flipping colors
- Probably best if this is an illusion provided by the display methods and the solver
  - Might be simpler if, under the hood, the position always flips
- Alternatively, can rename self.current_player_stones to self.first_player_stones
  - Then the min-max methods can flip colors, but these won't flip colors
*/

#[derive(PartialEq, Eq, Debug)]
pub enum GameState {
    InProgress,
    Ended(EndState),
}

#[derive(PartialEq, Eq, Debug)]
pub enum EndState {
    PriorPlayerWon,
    Drawn,
}

#[derive(PartialEq, Eq, Error, Debug)]
pub enum PlayColumnError {
    #[error("Cannot play, zero-based column index `{0}` does not exist. (expected < {max})", max = GAME_WIDTH)]
    ColumnDoesNotExist(u8),
    #[error("Cannot play, {}", Self::say_end_state_error(.0))]
    GameOver(EndState),
    #[error("Cannot play, column `{0}` is full")]
    ColumnFull(u8),
}

impl PlayColumnError {
    fn say_end_state_error(end_state: &EndState) -> &'static str {
        match end_state {
            EndState::PriorPlayerWon => "prior player has already won",
            EndState::Drawn => "game has already ended in a draw",
        }
    }
}


// Meant to be used when playing the game interactively
impl Position {
    pub fn get_game_state(&self) -> GameState {
        if Self::position_has_connect_four(self.current_player_stones ^ self.both_players_mask) {
            return GameState::Ended(EndState::PriorPlayerWon);
        }

        if self.moves_played == GAME_WIDTH * GAME_HEIGHT {
            return GameState::Ended(EndState::Drawn);
        }

        GameState::InProgress
    }

    pub fn column_is_open(&self, column: u8) -> bool {
        self.both_players_mask & bitmap::compute_top_of_playable_column_mask(column) == 0
    }

    pub fn play_column(&mut self, column: u8) -> Result<(), PlayColumnError> {
        if column >= GAME_WIDTH {
            return Err(PlayColumnError::ColumnDoesNotExist(column));
        }
        if let GameState::Ended(end_state) = self.get_game_state() {
            return Err(PlayColumnError::GameOver(end_state));
        }
        if !self.column_is_open(column) {
            return Err(PlayColumnError::ColumnFull(column));
        }

        self.current_player_stones ^= self.both_players_mask;
        self.both_players_mask |= self.both_players_mask + bitmap::compute_bottom_of_column_mask(column);
        self.moves_played += 1;

        Ok(())
    }

    fn position_has_connect_four(current_player_stones: Bitmap) -> bool {
        // Horizontal
        let mut pairs = current_player_stones & current_player_stones >> BITMAP_COLUMN_HEIGHT; // pairs has a 1 for all horizontal "connect 2"
        if pairs & pairs >> 2 * BITMAP_COLUMN_HEIGHT != 0 {
            return true;
        }

        // Up-Left-Down-Right Diagonal
        pairs = current_player_stones & current_player_stones >> UP_LEFT_DOWN_RIGHT_OFFSET;
        if pairs & pairs >> 2 * UP_LEFT_DOWN_RIGHT_OFFSET != 0 {
            return true;
        }

        // Down-Left-Up-Right Diagonal
        pairs = current_player_stones & current_player_stones >> DOWN_LEFT_UP_RIGHT_OFFSET;
        if pairs & pairs >> 2 * DOWN_LEFT_UP_RIGHT_OFFSET != 0 {
            return true;
        }

        // Vertical
        pairs = current_player_stones & current_player_stones >> 1;
        pairs & pairs >> 2 != 0
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{testing_constants::{BLANK_POSITION, COMPLEX_POSITION}, GAME_WIDTH};

    use super::*;

	#[test]
	fn test_prior_player_has_won() {
		// First show that neither player won the complex position prior to these moves
		assert!(!Position::position_has_connect_four(COMPLEX_POSITION.current_player_stones));
		assert!(!Position::position_has_connect_four(COMPLEX_POSITION.current_player_stones ^ COMPLEX_POSITION.both_players_mask));

		let mut vertical_complex_win = COMPLEX_POSITION;
		vertical_complex_win.play_column(1);
		assert!(vertical_complex_win.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));

		let mut horizontal_complex_win = COMPLEX_POSITION;
		horizontal_complex_win.play_column(0);
		assert!(horizontal_complex_win.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));

		let mut up_left_down_right_diagonal_win = COMPLEX_POSITION;
		up_left_down_right_diagonal_win.play_column(6);
		assert!(up_left_down_right_diagonal_win.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));

		let mut down_left_up_right_diagonal_win = COMPLEX_POSITION;
		down_left_up_right_diagonal_win.play_column(4);
		assert!(down_left_up_right_diagonal_win.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));
	}

	#[test]
    fn test_can_play_blank() {
        let all_true = [true; GAME_WIDTH as usize];
        let actual = std::array::from_fn(|col| BLANK_POSITION.column_is_open(col as u8));
        assert_eq!(all_true, actual);
    }

	#[test]
    fn test_can_play_complex() {
        let all_columns_except_one = [true, true, false, true, true, true, true];
        let actual = std::array::from_fn(|col| COMPLEX_POSITION.column_is_open(col as u8));
        assert_eq!(all_columns_except_one, actual);
    }

	#[test]
    fn test_is_winning_move_complex() {
        let expected = [true, true, false, false, true, false, true];
        let actual = std::array::from_fn(|col| {
            let mut complex_position = COMPLEX_POSITION;
            complex_position.play_column(col as u8);
            complex_position.get_game_state() == GameState::Ended(EndState::PriorPlayerWon)
        });
        assert_eq!(expected, actual);
    }

	#[test]
    fn test_blank_has_no_winning_moves() {
        let all_false = [false; GAME_WIDTH as usize];
        let actual = std::array::from_fn(|col| {
            let mut blank_position = BLANK_POSITION;
            blank_position.play_column(col as u8);
            blank_position.get_game_state() == GameState::Ended(EndState::PriorPlayerWon)
        });
        assert_eq!(all_false, actual);
    }

    #[test]
    fn test_play_col_throws_the_win() {
        let either_player_can_win = COMPLEX_POSITION;

        let mut plays_winning_column = either_player_can_win;
        plays_winning_column.play_column(0);
        assert!(plays_winning_column.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));

        let mut plays_losing_column = either_player_can_win;
        plays_losing_column.play_column(3);
        assert!(plays_losing_column.get_game_state() == GameState::InProgress);

        plays_losing_column.play_column(5);
        assert!(plays_losing_column.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));
    }
}

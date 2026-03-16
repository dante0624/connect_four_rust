use crate::board::{bitmap, Position, GAME_HEIGHT, GAME_WIDTH};
use crate::board::bitmap::{Bitmap, BITMAP_COLUMN_HEIGHT, BOTTOM_ROW_MASK, DOWN_LEFT_UP_RIGHT_OFFSET, PLAYABLE_BOARD_MASK, UP_LEFT_DOWN_RIGHT_OFFSET};

pub(crate) const MIN_SCORE: i8 = -((GAME_WIDTH * GAME_HEIGHT) as i8) / 2 + 3;
pub(crate) const MAX_SCORE: i8 = ((GAME_WIDTH * GAME_HEIGHT) as i8) / 2 - 3;

impl Position {
    pub const fn new_blank_game() -> Self {
        Position { current_player_stones: 0, both_players_mask: 0, moves_played: 0 }
    }

    fn get_playable_squares(&self) -> Bitmap {
        self.both_players_mask + BOTTOM_ROW_MASK & PLAYABLE_BOARD_MASK
    }

    fn compute_winning_positions(current_player_stones: Bitmap, both_players_mask: Bitmap) -> Bitmap {
        // Vertical
        let mut winning = current_player_stones << 1 & current_player_stones << 2 & current_player_stones << 3;

        // Horizontal
        // We will redefine pair many times, right now it means there are a pair of 1's to the left
        let mut pair = current_player_stones << BITMAP_COLUMN_HEIGHT & current_player_stones << 2 * BITMAP_COLUMN_HEIGHT;

        // Compute three to the left, then two left and one right
        winning |= pair & current_player_stones << 3 * BITMAP_COLUMN_HEIGHT;
        winning |= pair & current_player_stones >> BITMAP_COLUMN_HEIGHT;

        // Now, pair means there are a pair of 1's to the right
        pair = current_player_stones >> BITMAP_COLUMN_HEIGHT & current_player_stones >> 2 * BITMAP_COLUMN_HEIGHT;

        // Compute three to the right, then two right and one left
        winning |= pair & current_player_stones >> 3 * BITMAP_COLUMN_HEIGHT;
        winning |= pair & current_player_stones << BITMAP_COLUMN_HEIGHT;

        // Up-Left-Down-Right Diagonal
        // Now, pair means there are a pair of 1's up-left
        pair = current_player_stones << UP_LEFT_DOWN_RIGHT_OFFSET & current_player_stones << 2 * UP_LEFT_DOWN_RIGHT_OFFSET;

        // Compute 3 up-left, then two up-left and one down-right
        winning |= pair & current_player_stones << 3 * UP_LEFT_DOWN_RIGHT_OFFSET;
        winning |= pair & current_player_stones >> UP_LEFT_DOWN_RIGHT_OFFSET;

        // Now, pair means there are a pair of 1's down-right
        pair = current_player_stones >> UP_LEFT_DOWN_RIGHT_OFFSET & current_player_stones >> 2 * UP_LEFT_DOWN_RIGHT_OFFSET;

        // Compute 3 down-right, then two down-right and one up-left
        winning |= pair & current_player_stones >> 3 * UP_LEFT_DOWN_RIGHT_OFFSET;
        winning |= pair & current_player_stones << UP_LEFT_DOWN_RIGHT_OFFSET;

        // Down-Left-Up-Right Diagonal
        // Now, pair means there are a pair of 1's down-left
        pair = current_player_stones << DOWN_LEFT_UP_RIGHT_OFFSET & current_player_stones << 2 * DOWN_LEFT_UP_RIGHT_OFFSET;

        // Compute three down-left, then two down-left and one up-right
        winning |= pair & current_player_stones << 3 * DOWN_LEFT_UP_RIGHT_OFFSET;
        winning |= pair & current_player_stones >> DOWN_LEFT_UP_RIGHT_OFFSET;

        // Now, pair means there are a pair of 1's up-right
        pair = current_player_stones >> DOWN_LEFT_UP_RIGHT_OFFSET & current_player_stones >> 2 * DOWN_LEFT_UP_RIGHT_OFFSET;

        // Compute three up-right, then two up-right and one down-left
        winning |= pair & current_player_stones >> 3 * DOWN_LEFT_UP_RIGHT_OFFSET;
        winning |= pair & current_player_stones << DOWN_LEFT_UP_RIGHT_OFFSET;

        // Need to be a winning position, and there need to not already be a chip there
        winning & (PLAYABLE_BOARD_MASK ^ both_players_mask)
    }

    fn get_current_player_winning_positions(&self) -> Bitmap {
        Self::compute_winning_positions(self.current_player_stones, self.both_players_mask)
    }

    fn get_opponent_winning_positions(&self) -> Bitmap {
        Self::compute_winning_positions(self.current_player_stones ^ self.both_players_mask, self.both_players_mask)
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

    fn prior_player_has_won(&self) -> bool {
        Self::position_has_connect_four(self.current_player_stones ^ self.both_players_mask)
    }

    fn can_play(&self, column: u8) -> bool {
        self.both_players_mask & bitmap::compute_top_of_playable_column_mask(column) == 0
    }

    // Not actually used by the solver, but is used when running the game as web server
    pub fn play_column(&mut self, column: u8) {
        self.current_player_stones ^= self.both_players_mask;
        self.both_players_mask |= self.both_players_mask + bitmap::compute_bottom_of_column_mask(column);
        self.moves_played += 1;
    }

    // square is a bitmap with a single 1 where you want the new token to be placed
    // Must check beforehand that this move is valid (not floating or taken)
    // Used by the solver, as it is slightly faster than play_column,
    // assuming that the square and column are both already computed.
    fn play_square(&mut self, square: Bitmap) {
        self.current_player_stones ^= self.both_players_mask;
        self.both_players_mask |= square;
        self.moves_played += 1;
    }

    // Returns true if the player will win by playing in a current column
    // Note, if the position is already won (somehow the game didn't end) this still returns true
    fn is_winning_move(&self, column: u8) -> bool {
        let mut current_player_stones = self.current_player_stones;

        // Add a single 1 to the top of the desired column
        current_player_stones |= (self.both_players_mask + bitmap::compute_bottom_of_column_mask(column)) & bitmap::compute_playable_column_mask(column);

        Self::position_has_connect_four(current_player_stones)
    }

    // Return true if the currentPlayer can win on their current move
    fn can_win_next(&self) -> bool {
        self.get_current_player_winning_positions() & self.get_playable_squares() != 0
    }

    fn possible_non_losing_moves(&self) -> Bitmap {
        let mut possible_moves = self.get_playable_squares();
        let opponent_win_squares = self.get_opponent_winning_positions();
        let forced_moves = possible_moves & opponent_win_squares;

        if forced_moves != 0 {
            // This indicates that there are multiple forced moves, so we lose
            if forced_moves & (forced_moves - 1) != 0 {
                return 0;
            }
            possible_moves = forced_moves;
        }

        /* At this point, possibleMoves is one of two cases:
        Case 1: There are no forced moves, so possibleMoves is just all playable columns
        Case 2: There is a single forced move, so possibleMoves is that one move
        We now want to remove any moves which "build up" and reveal a new winning move to the opponent */
        possible_moves & !(opponent_win_squares >> 1)
    }

    fn move_score(&self, square: Bitmap) -> u8 {
        bitmap::count_ones_in_bitmap(Self::compute_winning_positions(square | self.current_player_stones, self.both_players_mask))
    }
}

#[cfg(test)]
mod tests {
    use crate::board::testing_constants::{BLANK_POSITION, COMPLEX_POSITION, UP_LEFT_DOWN_RIGHT_DIAGONAL, DOWN_LEFT_UP_RIGHT_DIAGONAL, HORIZONTAL, VERTICAL};

    use super::*;

	#[test]
	fn test_prior_player_has_won() {
		// First show that neither player won the complex position prior to these moves
		assert!(!Position::position_has_connect_four(COMPLEX_POSITION.current_player_stones));
		assert!(!Position::position_has_connect_four(COMPLEX_POSITION.current_player_stones ^ COMPLEX_POSITION.both_players_mask));

		let mut vertical_complex_win = COMPLEX_POSITION;
		vertical_complex_win.play_column(1);
		assert!(vertical_complex_win.prior_player_has_won());

		let mut horizontal_complex_win = COMPLEX_POSITION;
		horizontal_complex_win.play_column(0);
		assert!(horizontal_complex_win.prior_player_has_won());

		let mut up_left_down_right_diagonal_win = COMPLEX_POSITION;
		up_left_down_right_diagonal_win.play_column(6);
		assert!(up_left_down_right_diagonal_win.prior_player_has_won());

		let mut down_left_up_right_diagonal_win = COMPLEX_POSITION;
		down_left_up_right_diagonal_win.play_column(4);
		assert!(down_left_up_right_diagonal_win.prior_player_has_won());
	}

	#[test]
    fn test_can_play_blank() {
        let all_true = [true; GAME_WIDTH as usize];
        let actual = std::array::from_fn(|col| BLANK_POSITION.can_play(col as u8));
        assert_eq!(all_true, actual);
    }

	#[test]
    fn test_can_play_complex() {
        let all_columns_except_one = [true, true, false, true, true, true, true];
        let actual = std::array::from_fn(|col| COMPLEX_POSITION.can_play(col as u8));
        assert_eq!(all_columns_except_one, actual);
    }

	#[test]
    fn test_is_winning_move_complex() {
        let expected = [true, true, false, false, true, false, true];
        let actual = std::array::from_fn(|col| COMPLEX_POSITION.is_winning_move(col as u8));
        assert_eq!(expected, actual);
    }

	#[test]
    fn test_blank_has_no_winning_moves() {
        let all_false = [false; GAME_WIDTH as usize];
        let actual = std::array::from_fn(|col| BLANK_POSITION.is_winning_move(col as u8));
        assert_eq!(all_false, actual);
    }

    #[test]
    fn test_play_col_throws_the_win() {
        let mut either_player_can_win = COMPLEX_POSITION;
        assert!(either_player_can_win.is_winning_move(0));
        assert!(!either_player_can_win.is_winning_move(3));
        either_player_can_win.play_column(3);
        assert!(!either_player_can_win.prior_player_has_won());

        assert!(either_player_can_win.is_winning_move(5));
    }

    #[test]
    fn test_play_square_throws_the_win() {
        let mut either_player_can_win = COMPLEX_POSITION;
        assert!(either_player_can_win.is_winning_move(0));
        either_player_can_win.play_square(0x4000000);
        assert!(!either_player_can_win.prior_player_has_won());

        assert!(either_player_can_win.is_winning_move(5));
    }

    // Test can_win_next in each possible way
    #[test]
    fn test_can_win_next_vertical() {
        let mut vertical = VERTICAL;
        assert!(vertical.can_win_next());

        // block off the winning column
        vertical.play_column(1);
        vertical.play_column(0);
        assert!(!vertical.can_win_next());
    }

    #[test]
    fn test_can_win_next_horizontal() {
        let mut horizontal = HORIZONTAL;
        assert!(horizontal.can_win_next());

        // block off one winning column
        horizontal.play_column(3);
        horizontal.play_column(1);
        assert!(horizontal.can_win_next());

        // block off the other
        horizontal.play_column(1);
        horizontal.play_column(5);
        assert!(!horizontal.can_win_next());
    }

    #[test]
    fn test_can_win_next_up_left_down_right_diagonal() {
        let mut up_left_down_right_diagonal = UP_LEFT_DOWN_RIGHT_DIAGONAL;
        assert!(up_left_down_right_diagonal.can_win_next());

        // block off one winning column
        up_left_down_right_diagonal.play_column(0);
        up_left_down_right_diagonal.play_column(1);
        assert!(up_left_down_right_diagonal.can_win_next());

        // block off the other
        up_left_down_right_diagonal.play_column(6);
        up_left_down_right_diagonal.play_column(5);
        assert!(!up_left_down_right_diagonal.can_win_next());
    }

    #[test]
    fn test_can_win_next_down_right_up_left_diagonal() {
        let mut down_right_up_left_diagonal = DOWN_LEFT_UP_RIGHT_DIAGONAL;
        assert!(down_right_up_left_diagonal.can_win_next());

        // block off one winning column
        down_right_up_left_diagonal.play_column(6);
        down_right_up_left_diagonal.play_column(5);
        assert!(down_right_up_left_diagonal.can_win_next());

        // block off the other
        down_right_up_left_diagonal.play_column(0);
        down_right_up_left_diagonal.play_column(1);
        assert!(!down_right_up_left_diagonal.can_win_next());
    }

    // Test possible NonLosing Moves in several ways
    #[test]
    fn test_possible_non_losing_moves() {
        // Every Move is playable
        assert_eq!(0x40810204081, BLANK_POSITION.possible_non_losing_moves());

        // The playable moves are simply non-full columns
        let mut complex_position = COMPLEX_POSITION;
        complex_position.play_column(5);
        complex_position.play_column(6);
        assert_eq!(0x80204001008, complex_position.possible_non_losing_moves());

        // The only playable move blocks an opponent's winning move
        let mut vertical = VERTICAL;
        vertical.play_column(1);
        assert_eq!(0x8, vertical.possible_non_losing_moves());

        // Opponent can win in two spots, so we are done
        let mut horizontal = HORIZONTAL;
        horizontal.play_column(0);
        assert_eq!(0x0, horizontal.possible_non_losing_moves());

        // Do not build under an opponent and let them win
        horizontal.play_column(1);
        horizontal.play_column(1);
        horizontal.play_column(5);
        horizontal.play_column(5);
        horizontal.play_column(3);
        assert_eq!(0x42001000202, horizontal.possible_non_losing_moves());

        // You play a move to block a direct win, but that builds under a different win
        horizontal.play_column(6);
        horizontal.play_column(2);
        horizontal.play_column(6);
        horizontal.play_column(1);
        assert_eq!(0x0, horizontal.possible_non_losing_moves());

    }

    #[test]
    fn test_move_score() {
        let mut down_left_up_right_diagonal = DOWN_LEFT_UP_RIGHT_DIAGONAL;
        assert_eq!(2, down_left_up_right_diagonal.move_score(0x1));

        // Flip whose move it is by playing a useless move
        down_left_up_right_diagonal.play_square(0x1);

        // Check creating an alignment versus one that does not
        assert_eq!(0, down_left_up_right_diagonal.move_score(0x2));
        assert_eq!(1, down_left_up_right_diagonal.move_score(0x10000));
    }
}


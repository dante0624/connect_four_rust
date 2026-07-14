use crate::board::scored_position_iterator::{ScoredPositionIterator, OUTSIDE_IN_COLUMN_ORDER};
use crate::board::{bitmap, Bitmap, Position};
use crate::board::bitmap::{BITMAP_COLUMN_HEIGHT, BOTTOM_ROW_MASK, DOWN_LEFT_UP_RIGHT_OFFSET, PLAYABLE_BOARD_MASK, UP_LEFT_DOWN_RIGHT_OFFSET};

// Directly called by the null window search
impl Position {
    pub(crate) fn can_win_next(&self) -> bool {
        self.get_current_player_winning_positions() & self.get_playable_squares() != 0
    }

    pub(crate) fn all_moves_immediately_lose(&self) -> bool {
        self.possible_non_losing_moves() == 0
    }

    pub(crate) fn get_non_losing_child_positions(&self) -> impl Iterator<Item = Position> {
        let non_losing = self.possible_non_losing_moves();
        let mut sorted_child_position_iterator = ScoredPositionIterator::new();

        // Iterate through each column in columnOrder in reverse order, adding to the sorter
        // We go through in reverse order because liveSolverClasses.MoveSorter is a stack in the case of ties
        for col in OUTSIDE_IN_COLUMN_ORDER {
            let non_losing_square = non_losing & bitmap::compute_playable_column_mask(col);
            if non_losing_square != 0 {
                let score = self.move_score(non_losing_square);
                let mut child_position = self.clone();
                child_position.play_square(non_losing_square);
                sorted_child_position_iterator.add(child_position, score);
            }
        }
        sorted_child_position_iterator
    }
}

// Helper methods
impl Position {
    fn get_current_player_winning_positions(&self) -> Bitmap {
        Self::compute_winning_positions(self.current_player_stones, self.both_players_mask)
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

    fn get_playable_squares(&self) -> Bitmap {
        self.both_players_mask + BOTTOM_ROW_MASK & PLAYABLE_BOARD_MASK
    }

    fn move_score(&self, square: Bitmap) -> u8 {
        bitmap::count_ones_in_bitmap(Self::compute_winning_positions(square | self.current_player_stones, self.both_players_mask))
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


    fn get_opponent_winning_positions(&self) -> Bitmap {
        Self::compute_winning_positions(self.current_player_stones ^ self.both_players_mask, self.both_players_mask)
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
}


#[cfg(test)]
mod tests {
    use crate::board::scored_position_iterator::INSIDE_OUT_COLUMN_ORDER;
    use crate::board::testing_constants::{BLANK_POSITION, COMPLEX_POSITION, DOWN_LEFT_UP_RIGHT_DIAGONAL, HORIZONTAL, MIDDLE_COLUMN_FULL, PRE_HORIZONTAL, UP_LEFT_DOWN_RIGHT_DIAGONAL, VERTICAL};
    use crate::board::{EndState, GameState};

    use super::*;

    #[test]
    fn test_play_square_throws_the_win() {
        let either_player_can_win = COMPLEX_POSITION;

        let mut plays_winning_column = either_player_can_win;
        plays_winning_column.play_column(0);
        assert!(plays_winning_column.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));

        let mut plays_losing_square = either_player_can_win;
        // Corresponds to playing the 3rd column
        plays_losing_square.play_square(0x4000000);
        assert!(plays_losing_square.get_game_state() == GameState::InProgress);

        plays_losing_square.play_column(5);
        assert!(plays_losing_square.get_game_state() == GameState::Ended(EndState::PriorPlayerWon));
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
        assert!(!BLANK_POSITION.all_moves_immediately_lose());

        // The playable moves are simply non-full columns
        let mut complex_position = COMPLEX_POSITION;
        complex_position.play_column(5);
        complex_position.play_column(6);
        assert_eq!(0x80204001008, complex_position.possible_non_losing_moves());
        assert!(!complex_position.all_moves_immediately_lose());

        // The only playable move blocks an opponent's winning move
        let mut vertical = VERTICAL;
        vertical.play_column(1);
        assert_eq!(0x8, vertical.possible_non_losing_moves());
        assert!(!vertical.all_moves_immediately_lose());

        // Opponent can win in two spots, so we are done
        let mut horizontal = HORIZONTAL;
        horizontal.play_column(0);
        assert_eq!(0x0, horizontal.possible_non_losing_moves());
        assert!(horizontal.all_moves_immediately_lose());

        // Do not build under an opponent and let them win
        horizontal.play_column(1);
        horizontal.play_column(1);
        horizontal.play_column(5);
        horizontal.play_column(5);
        horizontal.play_column(3);
        assert_eq!(0x42001000202, horizontal.possible_non_losing_moves());
        assert!(!horizontal.all_moves_immediately_lose());

        // You play a move to block a direct win, but that builds under a different win
        horizontal.play_column(6);
        horizontal.play_column(2);
        horizontal.play_column(6);
        horizontal.play_column(1);
        assert_eq!(0x0, horizontal.possible_non_losing_moves());
        assert!(horizontal.all_moves_immediately_lose());

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

    #[test]
    fn test_blank_position_iterates_middle_out() {
        let expected_middle_out: Vec<Position> = INSIDE_OUT_COLUMN_ORDER.into_iter()
            .map(|col| {
                let mut first_move_played = BLANK_POSITION;
                first_move_played.play_column(col);
                first_move_played})
            .collect();
        let actual: Vec<Position> = BLANK_POSITION.get_non_losing_child_positions().collect();
        assert_eq!(expected_middle_out, actual);
    }

    #[test]
    fn test_child_positions_prefer_alignments() {
        let expected_column_order = [4, 1, 5, 0, 3, 2, 6];

        let expected_child_positions: Vec<Position> = expected_column_order.into_iter()
            .map(|col| {
                let mut fifth_move_played = PRE_HORIZONTAL;
                fifth_move_played.play_column(col);
                fifth_move_played})
            .collect();
        let actual: Vec<Position> = PRE_HORIZONTAL.get_non_losing_child_positions().collect();
        assert_eq!(expected_child_positions, actual);
    }

    #[test]
    fn test_child_positions_iterator_skips_full_columns() {
        let expected_column_order = [2, 4, 1, 5, 0, 6];

        let expected_child_positions: Vec<Position> = expected_column_order.into_iter()
            .map(|col| {
                let mut seventh_move_played = MIDDLE_COLUMN_FULL;
                seventh_move_played.play_column(col);
                seventh_move_played})
            .collect();
        let actual: Vec<Position> = MIDDLE_COLUMN_FULL.get_non_losing_child_positions().collect();
        assert_eq!(expected_child_positions, actual);
    }
}


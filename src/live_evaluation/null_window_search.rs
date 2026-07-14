use crate::board::Position;
use crate::live_evaluation::mtd::MAX_MOVES_PLAYED;
use crate::live_evaluation::transposition_table::{TableKey, TableResult, TranspositionTable};

/// Null window search itself is a specific instance of
/// Alpha-Beta pruning (negamax variant) where (alpha + 1 = beta).
pub(crate) struct NullWindowSearch<'a> {
    pub(crate) transposition_table: &'a mut TranspositionTable,
    pub(crate) node_count: &'a mut u64,
}

/*
 TODO: Add multithreading:
    Young Brothers Wait Concept (YBWC): https://www.chessprogramming.org/Young_Brothers_Wait_Concept
    Jamboree implementation of YBWC: https://www.chessprogramming.org/Jamboree
    Possibly, simplified Simplified ABDADA: https://www.tckerrigan.com/Chess/Parallel_Search/Simplified_ABDADA/

TODO: Consider making this search return a NonZeroU8 rather than an i8.
    It would use the same encoding as the transposition table's upper bound values
    1..=(MAX - MIN + 1) is a normal response.
    Then saving and getting upper bounds from the table requires no math.
    Also allows for returning an Option<NonZeroU8> if multithreading requires Option::empty.
    The negation trick is to now do (2 * MIDDLE_VAL - x) rather than just (-x).

TODO: Also consider consider swapping the upper bound encoding and the lower bound encoding.
    Will need to time it. Pretty sure it is more common to have an lower bound.
*/
impl NullWindowSearch<'_> {

    /// Evaluates the score a position using alpha beta, algorithm.
    /// But, it assumes a null window, so `beta == alpha + 1`
    ///
    /// # Assumptions:
    /// - Never called on a full board
    /// - No one has already won the game
    /// - The current player cannot simply win in one move
    ///
    /// # Normal Alpha-Beta Return Values:
    ///  - if `actual_score <= alpha`, then `actual_score <= return_value <= alpha`
    ///  - if `actual_score >= beta`, then `beta <= return_value <= actual_score`
    ///  - if `alpha < actual_score < beta`, then `return_value == actual_score`
    ///
    /// # This Function Return Values:
    ///  - if `actual_score <= alpha`, then `actual_score <= return_value <= alpha`
    ///  - if `actual_score > alpha`, then `alpha < return_value <= actual score`
    pub(crate) fn search(&mut self, position: Position, alpha: i8) -> i8 {
        *self.node_count += 1; 

        if position.all_moves_immediately_lose() {
            // Uses the assumption that the current player cannot win in one move
            // In this scenario the current player plays and then the opponent wins
            return -(MAX_MOVES_PLAYED - position.moves_played as i8) / 2;
        }

        // We also know that we cannot win on this move (assumption)
        // So best case is current, opponent, current wins
        let mut max = (MAX_MOVES_PLAYED - 1 - position.moves_played as i8) / 2;

        // Didn't return early, so we can prevent an opponent connect 4 on the next move
        // So worse case is current, opponent, current, opponent wins
        let mut min = -(MAX_MOVES_PLAYED - 2 - position.moves_played as i8) / 2;

        // Use the transposition table to get potentially tighter bounds
        let table_key = TableKey::from(position.get_key());
        match self.transposition_table.get(table_key) {
            TableResult::Miss => {},
            TableResult::LowerBound(lower_bound) => {
                if lower_bound > min { min = lower_bound }
            },
            TableResult::UpperBound(upper_bound) => {
                if upper_bound < max { max = upper_bound }
            },
        }

        if alpha >= max { return max; }
        if alpha < min { return min; }

        // The recursive cases
        for child_position in position.get_non_losing_child_positions() {
            let score = -self.search(child_position, -(alpha + 1));
            // Pruning case
            if score > alpha {
                self.transposition_table.put_lower_bound(table_key, score);
                return score;
            }
        }

        // Non-pruning case
        self.transposition_table.put_upper_bound(table_key, alpha);
        alpha
    }
}

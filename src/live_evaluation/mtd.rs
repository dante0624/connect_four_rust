use crate::board::{Position, GAME_HEIGHT, GAME_WIDTH};
use crate::live_evaluation::transposition_table::TranspositionTable;
use crate::live_evaluation::null_window_search::NullWindowSearch;

pub(crate) const MAXIMUM_SCORE: u8 = (GAME_WIDTH * GAME_HEIGHT + 1) / 2 - 3;
pub(crate) const NEGATED_MINIMUM_SCORE: u8 = GAME_WIDTH * GAME_HEIGHT / 2 - 3;

pub(crate) const MAX_MOVES_PLAYED: i8 = (GAME_WIDTH * GAME_HEIGHT) as i8;



/// Theoretically similar to https://www.chessprogramming.org/MTD(f) but without the f parameter
pub struct Solver {
    pub transposition_table: TranspositionTable,
    pub node_count: u64,
}

impl Solver {
    pub fn new() -> Self {
        Self { transposition_table: TranspositionTable::new(), node_count: 0 }
    }

    pub fn reset(&mut self) {
        self.transposition_table.reset();
        self.node_count = 0;
    }

    /// Evaluates the score a position
    ///
    /// # Assumptions:
    /// - No one has already won the game
    ///
    /// # Return Value:
    /// - Exact Score of the position
    pub fn solve(&mut self, position: Position) -> i8 {
        let mut null_window_search = NullWindowSearch {
            transposition_table: &mut self.transposition_table,
            node_count: &mut self.node_count,
        };

        // Check if we can win in one move on this turn, as Negamax will now assume that we cannot
        if position.can_win_next() {
            return (MAX_MOVES_PLAYED + 1 - position.moves_played as i8) / 2;
        }

        // Use Negamax and null window search
        // Comparable to using binary search, where we are searching for the true position score
        let mut min = -(MAX_MOVES_PLAYED - position.moves_played as i8) / 2;
        let mut max = (MAX_MOVES_PLAYED - position.moves_played as i8 + 1) / 2;


        while min < max {
            /* This is the true middle value between max and min
            We do this instead of (min + max) / 2, because we want the floor division to always
            round down to -inf, instead of towards 0.
            This is relevant if say min = -2, and max = -1. */
            let mut middle = min + (max - min) / 2;

            /* Problem is, if max = 21 and min = -21, then middle = 0
            We don't want to use this value in Negamax, because it will involve looking deeply
            We want to find quick winning paths or quick loosing paths
            So, if this happens, we set middle = max /2, or middle = min / 2 */
            if middle <= 0 && min / 2 < middle {
                middle = min / 2;
            }
            else if middle >= 0 && max / 2 > middle {
                middle = max / 2;
            }

            let result = null_window_search.search(position, middle);

            // This result tells us if the true position score is <= or >= middle
            if result <= middle {
                max = result;
            }
            else {
                min = result;
            }
        }
        // Loop ends when min = max = true score.
        min
    }
}


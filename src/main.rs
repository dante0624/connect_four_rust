mod board;
mod live_evaluation;

use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use board::Position;

use crate::{board::PlayColumnError, live_evaluation::Solver};

fn main() -> Result<()>{
    let mut solver = Solver::new();

    for line in std::io::stdin().lines() {
        let line = line?;
        let (position, expected_eval) = parse_line(&line)?;

        let start = SystemTime::now();
        let actual_eval = solver.solve(position);
        let end = SystemTime::now();

        if expected_eval != actual_eval {
            bail!(
                "Expected and actual evaluations differ. Input line: {}, actual_eval: {}, position: {}",
                line, actual_eval, position);
        }

        let duration_micros = end.duration_since(start)
            .with_context(|| format!("Got negative time duration after evaluating {}", line))?
            .as_micros();

        println!("{} {} {}", line, solver.node_count, duration_micros);

        solver.reset();
    }

    Ok(())
}


fn parse_line(line: &str) -> Result<(Position, i8)> {
    let mut position = Position::new_blank_game();

    let (columns, eval) = line.split_once(' ')
        .with_context(|| format!("Did not find a space in line: {}", line))?;

    for column_char in columns.chars() {
        let column: u8 = column_char.to_digit(10)
            .with_context(|| format!("Input character: {} could be be converted to a digit", column_char))?
            .try_into()
            .with_context(|| format!("Input character: {} could be be reduced from a u32", column_char))?;

        // File uses 1 based indexing
        position.play_column(column - 1)?;
    }

    let expected_eval = eval.parse::<i8>()?;

    Ok((position, expected_eval))
}

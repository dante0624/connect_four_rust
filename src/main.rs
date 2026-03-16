mod board;
use board::Position;

fn main() {
    let mut x = Position::new_blank_game();
    x.play_column(3);
    x.play_column(3);
    x.play_column(3);
    x.play_column(0);
    println!("{}", x);
    x.play_column(0);
    println!("");
    println!("{}", x);
}


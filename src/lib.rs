use std::io::{self, StdinLock, Stdout, Write};

use rand::prelude::*;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, style};

/// The upper and lower boundary char.
const HORZ_BOUNDARY: &'static str = "─";
/// The left and right boundary char.
const VERT_BOUNDARY: &'static str = "│";

/// The top-left corner
const TOP_LEFT_CORNER: &'static str = "┌";
/// The top-right corner
const TOP_RIGHT_CORNER: &'static str = "┐";
/// The bottom-left corner
const BOTTOM_LEFT_CORNER: &'static str = "└";
/// The bottom-right corner
const BOTTOM_RIGHT_CORNER: &'static str = "┘";

/// The empty cell
const EMPTY_CELL: &'static str = "·";

// Board size
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

// Tetromino block
const BLOCK: &'static str = "█";

// TODO: Simply
struct Point {
    x: usize,
    y: usize,
}

// Tetromino blocks
// Positioning:
// 00 01 02 03
// 10 11 12 13
// 20 21 22 23
// 30 31 32 33
// Each tetromino occupies 4 positions in the above sparse array.
// The struct stores xy for each block in the tetromino.
// Ref: https://en.wikipedia.org/wiki/Tetromino#One-sided_tetrominoes
// TODO: Maybe optimize to store as single int?
struct Tetromino {
    b1: Point,
    b2: Point,
    b3: Point,
    b4: Point,
}

impl Tetromino {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => Self::i(),
            1 => Self::o(),
            2 => Self::t(),
            3 => Self::j(),
            4 => Self::l(),
            5 => Self::s(),
            _ => Self::z(),
        }
    }

    fn i() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 0, y: 2 },
            b4: Point { x: 0, y: 3 },
        }
    }

    fn o() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 1, y: 0 },
            b4: Point { x: 1, y: 1 },
        }
    }

    fn t() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 0, y: 2 },
            b4: Point { x: 1, y: 1 },
        }
    }

    fn j() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 1 },
            b2: Point { x: 1, y: 1 },
            b3: Point { x: 2, y: 0 },
            b4: Point { x: 2, y: 1 },
        }
    }

    fn l() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 1, y: 0 },
            b3: Point { x: 2, y: 0 },
            b4: Point { x: 2, y: 1 },
        }
    }

    fn s() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 1 },
            b2: Point { x: 0, y: 2 },
            b3: Point { x: 1, y: 0 },
            b4: Point { x: 1, y: 1 },
        }
    }

    fn z() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 1, y: 1 },
            b4: Point { x: 1, y: 2 },
        }
    }
}

pub struct Game {
    board: Vec<Vec<String>>,
    score: i8,
    width: usize,
    height: usize,
    stdout: RawTerminal<Stdout>,
    stdin: Keys<StdinLock<'static>>,
}

impl Game {
    // default constructor
    pub fn default() -> Self {
        Self::new(BOARD_WIDTH, BOARD_HEIGHT)
    }

    // constructor
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            board: vec![vec![String::from(EMPTY_CELL); width]; height],
            score: 0,
            width: width,
            height,
            stdin: io::stdin().lock().keys(),
            stdout: io::stdout().into_raw_mode().unwrap(),
        }
    }

    // Print the game board.
    fn print_box(&mut self) {
        // Top row
        write!(self.stdout, "{}", TOP_LEFT_CORNER).unwrap();
        for _ in 0..(self.width * 2) {
            write!(self.stdout, "{}", HORZ_BOUNDARY).unwrap();
        }
        write!(self.stdout, "{}\n\r", TOP_RIGHT_CORNER).unwrap();

        // Body
        for _ in 0..self.height {
            write!(self.stdout, "{}", VERT_BOUNDARY).unwrap();
            for _ in 0..self.width {
                write!(self.stdout, "{} ", EMPTY_CELL).unwrap();
            }
            write!(self.stdout, "{}\n\r", VERT_BOUNDARY).unwrap();
        }

        // Bottom row
        write!(self.stdout, "{}", BOTTOM_LEFT_CORNER).unwrap();
        for _ in 0..(self.width * 2) {
            write!(self.stdout, "{}", HORZ_BOUNDARY).unwrap();
        }
        write!(self.stdout, "{}\n\r", BOTTOM_RIGHT_CORNER).unwrap();
    }

    // Move mouse to x, y.
    fn goto(&mut self, x: u16, y: u16) {
        write!(self.stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
    }

    // Write current score.
    fn print_score(&mut self) {
        // Move to bottom row
        self.goto(3, (self.height as u16) + 2);

        // Write score
        write!(
            self.stdout,
            "{} Score: {}{}",
            style::Bold,
            self.score,
            style::Reset
        )
        .unwrap();
    }

    // Init game screen.
    fn init_screen(&mut self) {
        // Clear display.
        write!(self.stdout, "{}", clear::All).unwrap();
        self.goto(1, 1);

        // Print box.
        self.print_box();

        // Print score.
        self.print_score();
    }

    fn insert(&mut self, t: Tetromino) {
        self.board[t.b1.x][t.b1.y] = String::from(BLOCK);
        self.board[t.b2.x][t.b2.y] = String::from(BLOCK);
        self.board[t.b3.x][t.b3.y] = String::from(BLOCK);
        self.board[t.b4.x][t.b4.y] = String::from(BLOCK);
    }

    fn draw(&mut self) {
        // Draw the board.
        for (j, row) in self.board.iter().enumerate() {
            // Goto line.
            write!(self.stdout, "{}", termion::cursor::Goto(2, (j as u16) + 2)).unwrap();

            // Write line.
            for cell in row.iter() {
                if cell == EMPTY_CELL {
                    write!(self.stdout, "{} ", EMPTY_CELL).unwrap();
                } else {
                    write!(self.stdout, "{}[]{}", color::Fg(color::Red), style::Reset).unwrap();
                }
            }
        }
    }

    // Start the game.
    pub fn run(&mut self) {
        self.init_screen();

        let t = Tetromino::random();
        self.insert(t);

        self.draw();
        self.goto(1, 30);
    }
}

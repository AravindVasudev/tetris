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
const EMPTY_CELL: &'static str = "· ";

// Board size
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

// Point struct
// The default board size is 20x10. x requires 5 bits & y requires 4 bits.
// So u8 is not an option. Given rust is efficient with structs, packing this
// into u16 would be a overkill.
// TODO: Maybe a different way to pack into u8?
struct Point {x: u8, y: u8}

// Tetromino blocks
// Positioning:
// 00 01 02 03
// 10 11 12 13
// 20 21 22 23
// 30 31 32 33
// Each tetromino occupies 4 positions in the above sparse array.
// The struct stores xy for each block in the tetromino.
// Ref: https://en.wikipedia.org/wiki/Tetromino#One-sided_tetrominoes
struct Tetromino {
    b1: Point,
    b2: Point,
    b3: Point,
    b4: Point,
    // Color is a trait. I got no idea what that is and instead of putting the
    // project on hold till I finish the book or keep going into my google
    // search hole, I'm hacking this to store the string.
    color: String,
}

impl Tetromino {
  // Get a random tetromino.
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

    // I tetromino.
    fn i() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 0, y: 2 },
            b4: Point { x: 0, y: 3 },
            color: format!("{}", color::Fg(color::Cyan)),
        }
    }

    // O tetromino.
    fn o() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 1, y: 0 },
            b4: Point { x: 1, y: 1 },
            color: format!("{}", color::Fg(color::Yellow)),
        }
    }

    // T tetromino.
    fn t() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 0, y: 2 },
            b4: Point { x: 1, y: 1 },
            color: format!("{}", color::Fg(color::Magenta)),
        }
    }

    // J tetromino.
    fn j() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 1 },
            b2: Point { x: 1, y: 1 },
            b3: Point { x: 2, y: 0 },
            b4: Point { x: 2, y: 1 },
            color: format!("{}", color::Fg(color::Blue)),
        }
    }

    // L tetromino.
    fn l() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 1, y: 0 },
            b3: Point { x: 2, y: 0 },
            b4: Point { x: 2, y: 1 },
            color: format!("{}", color::Fg(color::Rgb(255, 165, 0))),
        }
    }

    // S tetromino.
    fn s() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 1 },
            b2: Point { x: 0, y: 2 },
            b3: Point { x: 1, y: 0 },
            b4: Point { x: 1, y: 1 },
            color: format!("{}", color::Fg(color::Green)),
        }
    }

    // Z tetromino.
    fn z() -> Self {
        Tetromino {
            b1: Point { x: 0, y: 0 },
            b2: Point { x: 0, y: 1 },
            b3: Point { x: 1, y: 1 },
            b4: Point { x: 1, y: 2 },
            color: format!("{}", color::Fg(color::Red)),
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
                write!(self.stdout, "{}", EMPTY_CELL).unwrap();
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
        let block = format!("{}[]{}", t.color, style::Reset);
        self.board[t.b1.x as usize][t.b1.y as usize] = block.clone();
        self.board[t.b2.x as usize][t.b2.y as usize] = block.clone();
        self.board[t.b3.x as usize][t.b3.y as usize] = block.clone();
        self.board[t.b4.x as usize][t.b4.y as usize] = block.clone();
    }

    fn draw(&mut self) {
        // Draw the board.
        for (j, row) in self.board.iter().enumerate() {
            // Goto line.
            write!(self.stdout, "{}", termion::cursor::Goto(2, (j as u16) + 2)).unwrap();

            // Write line.
            for cell in row.iter() {
              write!(self.stdout, "{}", cell).unwrap();
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

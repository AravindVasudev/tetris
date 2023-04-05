use std::io::{self, StdinLock, Stdout, Write};
use std::ops;

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
// Edit: On retrospect, this needs signed ints. Translation could yield -ve num.
// Maybe given we only translate one position at a time, an optimization would
// be to check if offset < 0 and (x or y) == 0 for invalid offset. That way, I
// can still use u8.
// TODO: Maybe a different way to pack into u8?
struct Point {
    x: i16,
    y: i16,
}

impl ops::AddAssign<&Point> for Point {
    fn add_assign(&mut self, other: &Point) {
        *self = Self{
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
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
struct Tetromino {
    blocks: [Point; 4],
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
            blocks: [
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: 0, y: 3 },
            ],
            color: format!("{}", color::Fg(color::Cyan)),
        }
    }

    // O tetromino.
    fn o() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 1, y: 0 },
                Point { x: 1, y: 1 },
            ],
            color: format!("{}", color::Fg(color::Yellow)),
        }
    }

    // T tetromino.
    fn t() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: 1, y: 1 },
            ],
            color: format!("{}", color::Fg(color::Magenta)),
        }
    }

    // J tetromino.
    fn j() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 1 },
                Point { x: 1, y: 1 },
                Point { x: 2, y: 0 },
                Point { x: 2, y: 1 },
            ],
            color: format!("{}", color::Fg(color::Blue)),
        }
    }

    // L tetromino.
    fn l() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 2, y: 0 },
                Point { x: 2, y: 1 },
            ],
            color: format!("{}", color::Fg(color::Rgb(255, 165, 0))),
        }
    }

    // S tetromino.
    fn s() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 1 },
                Point { x: 0, y: 2 },
                Point { x: 1, y: 0 },
                Point { x: 1, y: 1 },
            ],
            color: format!("{}", color::Fg(color::Green)),
        }
    }

    // Z tetromino.
    fn z() -> Self {
        Tetromino {
            blocks: [
                Point { x: 0, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 1, y: 1 },
                Point { x: 1, y: 2 },
            ],
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
        let format = format!("{}[]{}", t.color, style::Reset);

        for block in t.blocks.iter() {
            self.board[block.x as usize][block.y as usize] = format.clone();
        }
    }

    // Translate tetromino.
    fn translate(&mut self, t: &mut Tetromino, offset: Point) {
        // Don't translate if any block fails bound check.
        // TODO: extract validation into a fn.
        for block in t.blocks.iter() {
            let new_x = block.x + offset.x;
            let new_y = block.y + offset.y;

            if new_x < 0 || new_x >= (self.height as i16) || new_y < 0 || new_y >= (self.width as i16) {
                return;
            }
        }

        // Translate
        for i in 0..t.blocks.len() {
            t.blocks[i] += &offset;
        }
    }

    // Translate tetromino left.
    fn left(&mut self, t: &mut Tetromino) {
        self.translate(t, Point { x: 0, y: -1 });
    }

    // Translate tetromino right.
    fn right(&mut self, t: &mut Tetromino) {
        self.translate(t, Point { x: 0, y: 1 });
    }

    // Translate tetromino down.
    fn down(&mut self, t: &mut Tetromino) {
        self.translate(t, Point { x: 1, y: 0 });
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

        let mut t = Tetromino::random();
        self.left(&mut t);
        self.right(&mut t);
        self.right(&mut t);
        self.down(&mut t);
        self.insert(t);

        self.draw();
        self.goto(1, 30);
    }
}

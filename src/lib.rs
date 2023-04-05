
use std::io::{self, Write, Stdout, StdinLock};

use termion::input::{TermRead, Keys};
use termion::{clear, style};

use termion::raw::{IntoRawMode, RawTerminal};

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

pub struct Game {
  board: Vec<Vec<u8>>,
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
      board: vec![vec![0; width]; height],
      score: 0,
      width: width,
      height,
      stdin: io::stdin().lock().keys(),
      stdout: io::stdout().into_raw_mode().unwrap(),
    }
  }

  fn print_box(&mut self) {
    // Top row
    write!(self.stdout, "{}",  TOP_LEFT_CORNER).unwrap();
    for _ in 0..(self.width * 2) {
      write!(self.stdout, "{}",  HORZ_BOUNDARY).unwrap();
    }
    write!(self.stdout, "{}\n\r",  TOP_RIGHT_CORNER).unwrap();

    // Body
    for _ in 0..self.height {
      write!(self.stdout, "{}",  VERT_BOUNDARY).unwrap();
      for _ in 0..self.width {
        write!(self.stdout, "{} ",  EMPTY_CELL).unwrap();
      }
      write!(self.stdout, "{}\n\r",  VERT_BOUNDARY).unwrap();
    }

    // Bottom row
    write!(self.stdout, "{}",  BOTTOM_LEFT_CORNER).unwrap();
    for _ in 0..(self.width * 2) {
      write!(self.stdout, "{}",  HORZ_BOUNDARY).unwrap();
    }
    write!(self.stdout, "{}\n\r",  BOTTOM_RIGHT_CORNER).unwrap();


  }

  // Move mouse to x, y.
  fn goto(&mut self, x: u16, y: u16) {
    write!(self.stdout, "{}",  termion::cursor::Goto(x, y)).unwrap();
  }

  fn print_score(&mut self) {
    // Move to bottom row
    self.goto(3, (self.height as u16) + 2);
    
    // Write score
    write!(self.stdout, "{} Score: {}{}", style::Bold, self.score, style::Reset).unwrap();
  }

  fn init_screen(&mut self) {
    // Clear display.
    write!(self.stdout, "{}", clear::All).unwrap();
    self.goto(1, 1);

    // Print box.
    self.print_box();

    // Print score.
    self.print_score();

    // TODO: remove this.
    self.goto(1, 30);
  }

  pub fn run(&mut self) {
    self.init_screen();
  }
}
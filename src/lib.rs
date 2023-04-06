use std::io::{self, StdinLock, Stdout, Write};
use std::time::{Duration, Instant};
use std::{ops, thread};

use rand::prelude::*;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{async_stdin, clear, color, cursor, style, AsyncReader};

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

const FRAME_RATE: u8 = 60; // 60 FPS
const FALL_RATE_MS: u128 = 1000; // 1 sec

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
        *self = Self {
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
    stdin: Keys<AsyncReader>,
    falling: Option<Tetromino>,
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
            stdin: async_stdin().keys(),
            stdout: io::stdout().into_raw_mode().unwrap(),
            falling: None,
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
        // Hide cursor
        write!(self.stdout, "{}", cursor::Hide).unwrap();

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
    fn translate(t: &mut Tetromino, offset: Point, w: usize, h: usize) {
        // Don't translate if any block fails bound check.
        // TODO: extract validation into a fn.
        for block in t.blocks.iter() {
            let new_x = block.x + offset.x;
            let new_y = block.y + offset.y;

            if new_x < 0 || new_x >= (w as i16) || new_y < 0 || new_y >= (h as i16) {
                return;
            }
        }

        // Translate
        for i in 0..t.blocks.len() {
            t.blocks[i] += &offset;
        }
    }

    // Translate tetromino left.
    fn left(t: &mut Tetromino, w: usize, h: usize) {
        Self::translate(t, Point { x: -1, y: 0 }, w, h);
    }

    // Translate tetromino right.
    fn right(t: &mut Tetromino, w: usize, h: usize) {
        Self::translate(t, Point { x: 1, y: 0 }, w, h);
    }

    // Translate tetromino down.
    fn down(t: &mut Tetromino, w: usize, h: usize) {
        Self::translate(t, Point { x: 0, y: 1 }, w, h);
    }

    fn rotate_counter_clockwise(t: &mut Tetromino, w: usize, h: usize) {
        // Center piece. So, here's the thing -- we need some center point to
        // rotate around. For now, we just assume the 2nd piece to the rotation
        // center. There is 4 blocks per tetromino now this works but maybe
        // consider a size-agnostic way?
        let cx = t.blocks[1].x;
        let cy = t.blocks[1].y;

        // Validate if rotation is within the board.
        // yeah, yeah, I know having duplicate checks within validate and update.
        // And I should probably create a transformed tetromino, validate, and
        // if that passes replace the ref.
        // TODO: Maybe do this? DRY ftw!
        for block in t.blocks.iter() {
            // To y'all who say programmer don't need math, check this out.
            // So, lets go into what's going on. We know basic geometry.
            // For a point (x, y) with center (0, 0), the counter-clockwise
            // rotation would be (-y, x). I'm basically using this here.
            // First, offset (x, y) by (-cx, -cy) a.k.a the center piece to
            // get the block relative to a (0, 0) center. Then do the rotation,
            // i.e., (-y, x) and then add back the offset (cx, cy).
            let x = block.x - cx;
            let y = block.y - cy;
            let new_x = -y + cx;
            let new_y = x + cy;

            if new_x < 0 || new_x >= (w as i16) || new_y < 0 || new_y >= (h as i16) {
                return;
            }
        }

        // Rotate
        for i in 0..t.blocks.len() {
            let x = t.blocks[i].x - cx;
            let y = t.blocks[i].y - cy;

            t.blocks[i].x = -y + cx;
            t.blocks[i].y = x + cy;
        }
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

        // Reset cursor
        write!(self.stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    }

    fn draw_falling(&mut self) {
        if let Some(t) = self.falling.as_ref() {
            for block in t.blocks.iter() {
                // Goto position.
                write!(
                    self.stdout,
                    "{}",
                    termion::cursor::Goto((block.x as u16) * 2 + 2, (block.y as u16) + 2)
                )
                .unwrap();

                // Draw block.
                write!(self.stdout, "{}[]{}", t.color, style::Reset).unwrap();
            }
        }
    }

    // Start the game.
    pub fn run(&mut self) {
        self.init_screen();

        let mut old_time = Instant::now();
        'game: loop {
            // Init block
            if let Some(t) = self.falling.as_mut() {
                // This block handles the tetrominos falling. This works independent of the current frame rate.
                // Maybe there are better ways of handling this but hey, this works.
                if old_time.elapsed().as_millis() >= FALL_RATE_MS {
                    // fall.
                    Self::down(t, self.width, self.height);

                    // Reset clock.
                    old_time = Instant::now();
                }

                // Next move.
                // Bad design aravind, bad design.
                // users can't quit if there is no block!
                match self.stdin.next() {
                    Some(Ok(key)) => {
                        match key {
                            Key::Char('q') => break 'game, // Quit
                            Key::Char('a') | Key::Left => Self::left(t, self.width, self.height),
                            Key::Char('s') | Key::Down => Self::down(t, self.width, self.height),
                            Key::Char('d') | Key::Right => Self::right(t, self.width, self.height),
                            Key::Char('w') | Key::Up => Self::rotate_counter_clockwise(t, self.width, self.height),
                            _ => (),
                        };
                    }
                    _ => {}
                }
            } else {
                // Create a new falling piece if there isn't one currently.
                let mut t = Tetromino::random();

                // center it.
                Self::translate(
                    &mut t,
                    Point {
                        x: ((self.width / 2) as i16) - 1,
                        y: 0,
                    },
                    self.width,
                    self.height,
                );

                self.falling = Some(t);
            }

            // All the game checks here.
            

            // Draw board.
            self.draw();

            // Draw falling.
            self.draw_falling();

            self.stdout.flush().unwrap();

            thread::sleep(Duration::from_millis(1000 / (FRAME_RATE as u64)));
            // break;
        }

        self.goto(0, 30);
    }
}

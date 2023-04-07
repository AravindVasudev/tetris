use std::collections::HashSet;
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
const FALL_RATE_MS: u128 = 600; // 0.6 sec

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

// GameState represents all the state the game can be in.
// Yeah, yeah, I know. Ideally, I'd like to have a start screen state,
// pause state, maybe win? (but what really is winning in tetris?).
enum GameState {
    PLAY,
    LOSE,
}

pub struct Game {
    // Bad design aravind, very bad.
    // Now that the board is a str, every freaking el is on the heap and every
    // comparison is expensive. Each cell stores two info: occupied and color.
    // Could have compressed into a single u8.
    board: Vec<Vec<String>>,
    score: i64,
    width: usize,
    height: usize,
    stdout: RawTerminal<Stdout>,
    stdin: Keys<AsyncReader>,
    falling: Option<Tetromino>,
    state: GameState,
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
            state: GameState::PLAY,
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

    fn insert_falling(&mut self) {
        if let Some(t) = self.falling.as_ref() {
            let format = format!("{}[]{}", t.color, style::Reset);
            for block in t.blocks.iter() {
                self.board[block.y as usize][block.x as usize] = format.clone();
            }
        }

        self.falling = None; // The board absorbs the falling piece.
    }

    // Translate tetromino.
    // ik, ik, w, h, and board is repeated params. And this can be moved to the tetromino struct.
    // thenks for you opinion.
    fn translate(
        t: &mut Tetromino,
        offset: Point,
        w: usize,
        h: usize,
        board: &Vec<Vec<String>>,
    ) -> bool {
        // Don't translate if any block fails bound check.
        // TODO: extract validation into a fn.
        for block in t.blocks.iter() {
            let new_x = block.x + offset.x;
            let new_y = block.y + offset.y;

            if new_x < 0
                || new_x >= (w as i16)
                || new_y < 0
                || new_y >= (h as i16)
                || board[new_y as usize][new_x as usize] != EMPTY_CELL
            {
                return false;
            }
        }

        // Translate
        for i in 0..t.blocks.len() {
            t.blocks[i] += &offset;
        }

        return true;
    }

    // Translate tetromino left.
    fn left(t: &mut Tetromino, w: usize, h: usize, board: &Vec<Vec<String>>) -> bool {
        Self::translate(t, Point { x: -1, y: 0 }, w, h, board)
    }

    // Translate tetromino right.
    fn right(t: &mut Tetromino, w: usize, h: usize, board: &Vec<Vec<String>>) -> bool {
        Self::translate(t, Point { x: 1, y: 0 }, w, h, board)
    }

    // Translate tetromino down.
    fn down(t: &mut Tetromino, w: usize, h: usize, board: &Vec<Vec<String>>) -> bool {
        Self::translate(t, Point { x: 0, y: 1 }, w, h, board)
    }

    fn rotate_counter_clockwise(t: &mut Tetromino, w: usize, h: usize, board: &Vec<Vec<String>>) {
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
            // To y'all who say programmers don't need math, check this out.
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

            if new_x < 0
                || new_x >= (w as i16)
                || new_y < 0
                || new_y >= (h as i16)
                || board[new_y as usize][new_x as usize] != EMPTY_CELL
            {
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

    // clears completed lines and updates score.
    // Scoring mechanism:
    //  For now, each completed line adds 100 pts.
    // Each press of the down key and make the fall faster adds 1 pt.
    // TODO: clearing multiple lines together should have score multiple.
    fn clear_completed_lines(&mut self) {
        for i in (0..self.height).rev() {
            // Check if the whole row is occupied.
            let mut occupied = 0;
            for j in 0..self.width {
                if self.board[i][j] != EMPTY_CELL {
                    occupied += 1;
                }
            }

            // If yes, update score.
            if occupied == self.width {
                self.score += 100;
            }

            // Clear row if its all occupied or all free.
            if occupied == 0 || occupied == self.width {
                // If not row above, just clear the row.
                if i == 0 {
                    for j in 0..self.width {
                        self.board[i][j] = String::from(EMPTY_CELL);
                    }
                } else {
                    // fallllll
                    for j in 0..self.width {
                        self.board[i][j] = self.board[i - 1][j].clone();
                        self.board[i - 1][j] = String::from(EMPTY_CELL);
                    }
                }
            }
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

    // draw the falling piece.
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

    // Draw game over
    fn draw_game_over(&mut self) {
        if matches!(self.state, GameState::LOSE) {
            // Goto middle
            self.goto(4, (self.width / 2 + 2) as u16);

            // Draw
            write!(
                self.stdout,
                "{}{}GAME OVER ☹️{}",
                style::Bold,
                color::Fg(color::Red),
                color::Fg(color::Reset)
            )
            .unwrap();
        }
    }

    // Validate if done falling.
    fn done_falling(&self) -> bool {
        if let Some(t) = &self.falling.as_ref() {
            // If any of the blocks sit on another block/ground, the block is done
            // falling.
            for block in t.blocks.iter() {
                if block.y >= (self.height as i16) - 1
                    || self.board[(block.y + 1) as usize][block.x as usize] != EMPTY_CELL
                {
                    return true;
                }
            }
        }

        return false;
    }

    fn update_game_state(&mut self) {
        // let's keep it stupid simple -- if board[0][center] is occupied, it's
        // game over. Is it hacky if it works?
        if self.board[0][(self.width / 2) - 1] != EMPTY_CELL
            || self.board[1][(self.width / 2) - 1] != EMPTY_CELL
        {
            self.state = GameState::LOSE;
        }
    }

    // Start the game.
    pub fn run(&mut self) {
        self.init_screen();

        let mut old_time = Instant::now();
        'game: loop {
            // Game Over :(
            if matches!(self.state, GameState::LOSE) {
                self.draw_game_over();
                break;
            }

            if let Some(t) = self.falling.as_mut() {
                // This block handles the tetrominos falling. This works independent of the current frame rate.
                // Maybe there are better ways of handling this but hey, this works.
                if old_time.elapsed().as_millis() >= FALL_RATE_MS {
                    // fall.
                    Self::down(t, self.width, self.height, &self.board);

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
                            Key::Char('a') | Key::Left => {
                                Self::left(t, self.width, self.height, &self.board);
                            }
                            Key::Char('s') | Key::Down => {
                                Self::down(t, self.width, self.height, &self.board);
                                self.score += 1;
                            }
                            Key::Char('d') | Key::Right => {
                                Self::right(t, self.width, self.height, &self.board);
                            }
                            Key::Char('w') | Key::Up => Self::rotate_counter_clockwise(
                                t,
                                self.width,
                                self.height,
                                &self.board,
                            ),
                            _ => (),
                        };
                    }
                    _ => {}
                }
            } else {
                // Create a new falling piece if there isn't one currently.
                let mut t = Tetromino::random();

                // center it.
                // If center fails since the piece overlaps, the game is over.
                if !Self::translate(
                    &mut t,
                    Point {
                        x: ((self.width / 2) as i16) - 1,
                        y: 0,
                    },
                    self.width,
                    self.height,
                    &self.board,
                ) {
                    self.state = GameState::LOSE;
                }

                self.falling = Some(t);
            }

            // All the game checks here.
            // Check if done falling, i.e., touches the ground or another block.
            if self.done_falling() {
                self.insert_falling();
            }

            // Clear completed lines
            self.clear_completed_lines();

            // Draw board.
            self.draw();

            // Draw score
            self.print_score();

            // Draw falling.
            self.draw_falling();

            // Flush stdout
            self.stdout.flush().unwrap();

            // Update game state
            self.update_game_state();

            // Maintain frame rate.
            thread::sleep(Duration::from_millis(1000 / (FRAME_RATE as u64)));
        }

        // Move cursor out of the board and show cursor.
        // If not, the terminal clears the board.
        self.goto(0, (self.height as u16) + 3);
        write!(self.stdout, "{}", cursor::Show).unwrap();
    }
}

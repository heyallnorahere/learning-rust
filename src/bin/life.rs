use bitvector::BitVector;
use console::Term;
use std::env::args;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write, stdin};
use std::sync::mpsc::{TryRecvError, channel};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub struct Board {
    data: BitVector,
    rows: usize,
    columns: usize,
}

impl Board {
    pub fn new(rows: usize, columns: usize) -> Board {
        Board {
            data: BitVector::new(rows * columns),
            rows: rows,
            columns: columns,
        }
    }

    fn of_term_size(term: &Term) -> Board {
        let size = term.size();
        Board::new(size.0.into(), size.1.into())
    }

    fn is_current(&self, term: &Term) -> bool {
        let size = term.size();
        self.rows == size.0.into() && self.columns == size.1.into()
    }

    pub fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.columns || y >= self.rows {
            None
        } else {
            Some(y * self.columns + x)
        }
    }

    pub fn get_position(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.rows * self.columns {
            None
        } else {
            Some((index % self.columns, index / self.columns))
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.get_index(x, y)
            .map_or(false, |index| self.data.contains(index))
    }

    pub fn set(&mut self, x: usize, y: usize) -> Option<bool> {
        self.get_index(x, y).map(|index| self.data.insert(index))
    }

    pub fn unset(&mut self, x: usize, y: usize) -> Option<bool> {
        self.get_index(x, y).map(|index| self.data.remove(index))
    }
}

fn render(output: &mut Term, board: &Board) {
    output.clear_screen().unwrap();

    for index in &board.data {
        // we know that index is valid
        let pos = board.get_position(index).unwrap();

        output.move_cursor_to(pos.0, pos.1).unwrap();
        output.write("@".as_bytes()).unwrap();
    }

    output.flush().unwrap();
}

struct CellStatus {
    alive: bool,
    neighbors: usize,
}

fn check_cell(x: usize, y: usize, board: &Board) -> CellStatus {
    let mut status = CellStatus {
        alive: false,
        neighbors: 0,
    };

    for delta_x in -1..=1 {
        let neighbor_x = match x.checked_add_signed(delta_x) {
            None => continue,
            Some(val) => val,
        };

        for delta_y in -1..=1 {
            let neighbor_y = match y.checked_add_signed(delta_y) {
                None => continue,
                Some(val) => val,
            };

            if board.get(neighbor_x, neighbor_y) {
                if neighbor_x == x && neighbor_y == y {
                    status.alive = true;
                } else {
                    status.neighbors += 1;
                }
            }
        }
    }

    status
}

fn will_survive(neighbors: usize) -> bool {
    if neighbors < 2 {
        false // underpopulation
    } else if neighbors > 3 {
        false // overpopulation
    } else {
        true
    }
}

fn will_live(status: &CellStatus) -> bool {
    if status.alive {
        will_survive(status.neighbors)
    } else {
        // reproduction
        status.neighbors == 3
    }
}

fn evaluate_board(new: &mut Board, old: &Board) {
    for y in 0..new.rows {
        for x in 0..new.columns {
            let status = check_cell(x, y, &old);

            if will_live(&status) {
                new.set(x, y);
            } else {
                new.unset(x, y);
            }
        }
    }
}

fn can_reuse_board(old: &Option<Board>, output: &Term) -> bool {
    match old {
        None => false,
        Some(board) => board.is_current(&output),
    }
}

#[derive(Debug)]
enum CoordinateParseError {
    InvalidFormat,
}

impl Display for CoordinateParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Error for CoordinateParseError {
    // nothing?
}

fn parse_coordinate(data: &str) -> Result<(usize, usize), Box<dyn Error>> {
    let ordinals: Vec<&str> = data.trim().split(',').collect();
    if ordinals.len() != 2 {
        return Err(Box::new(CoordinateParseError::InvalidFormat));
    }

    let mut coordinate = Vec::new();
    for parsed in ordinals.iter().map(|ordinal| ordinal.parse()) {
        match parsed {
            Ok(val) => coordinate.push(val),
            Err(err) => return Err(Box::new(err)),
        }
    }

    Ok((coordinate[0], coordinate[1]))
}

fn load_board(src: &mut dyn Read, board: &mut Board) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    src.read_to_string(&mut input)?;

    let lines = input.trim().lines();
    for line in lines {
        let coordinate = parse_coordinate(line.trim())?;
        board.set(coordinate.0, coordinate.1);
    }

    Ok(())
}

fn parse_args(args: &Vec<String>, board: &mut Board) -> Result<(), Box<dyn Error>> {
    if args.len() <= 1 {
        return Ok(());
    }

    let identifier = &args[1];
    let input: &mut dyn Read = match identifier.as_str() {
        "--" => &mut stdin(),
        _ => &mut File::open(identifier)?,
    };

    load_board(input, board)
}

fn main() {
    let mut term = Term::stdout();
    let mut current = Board::of_term_size(&term);

    let args: Vec<String> = args().collect();
    parse_args(&args, &mut current).unwrap();

    term.hide_cursor().unwrap();

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();

    let mut old: Option<Board> = None;
    let mut t0 = SystemTime::now();

    loop {
        match rx.try_recv() {
            Ok(_) => break,
            Err(err) => match err {
                TryRecvError::Empty => (),
                TryRecvError::Disconnected => panic!("Channel was disconnected!"),
            },
        }

        let mut work = match can_reuse_board(&old, &term) {
            true => old.take().unwrap(),
            false => Board::of_term_size(&term),
        };

        evaluate_board(&mut work, &current);

        old = Some(current);
        current = work;

        render(&mut term, &current);

        let t1 = SystemTime::now();
        let delta = t1.duration_since(t0).unwrap();
        t0 = t1;

        let min_delta = Duration::from_millis(50);
        if delta < min_delta {
            sleep(min_delta - delta);
        }
    }

    term.show_cursor().unwrap();
}

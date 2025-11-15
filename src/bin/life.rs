use bitvector::BitVector;
use console::Term;
use std::io::Write;
use std::sync::mpsc::{channel, TryRecvError};
use std::time::{Duration, SystemTime};
use std::thread::sleep;

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
        Board::new(size.1.into(), size.0.into())
    }

    fn is_current(&self, term: &Term) -> bool {
        let size = term.size();
        self.rows == size.1.into() && self.columns == size.0.into()
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
        self.get_index(x, y).map_or(false, |index| self.data.contains(index))
    }

    pub fn set(&mut self, x: usize, y: usize) -> Option<bool> {
        self.get_index(x, y).map(|index| self.data.insert(index))
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

fn evaluate_board(new: &mut Board, old: &Board) {
    // todo: implement game of life
}

fn can_reuse_board(old: &Option<Board>, output: &Term) -> bool {
    match old {
        None => false,
        Some(board) => board.is_current(&output),
    }
}

fn main() {
    let mut term = Term::stdout();
    let mut current = Board::of_term_size(&term);

    // todo: load

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
            }
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

        let min_delta = Duration::from_millis(16);
        if delta < min_delta {
            sleep(min_delta - delta);
        }
    }

    term.show_cursor().unwrap();
}

use std::ascii::AsciiExt;
use std::collections::BTreeSet;
use std::fmt;
use std::iter::{Iterator, IntoIterator};
use std::ops::Index;

use bit_vec::BitVec;

use error::Error;

const DIRECTIONS: [(isize, isize); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

pub struct Board {
    len: usize,
    board: Vec<u8>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Board:\t")?; 
        for i in 0..self.len() {
            for j in 0..self.len() {
                let idx = self.len() * i + j;
                write!(f, "{:?}", self.board[idx] as char)?;
            }
            write!(f, "\n\t")?;
        }
        Ok(())
    }
}

impl Board {
    pub fn parse(raw: &str) -> Result<Board, Error> {
        assert!(raw.is_ascii());

        let len = raw.lines().count();
        if len < 3 {
            return Err(Error::BoardSize("board must be at least 3 x 3"))
        }
        let mut board = Vec::with_capacity(len * len);

        for line in raw.lines() {
            if line.as_bytes().len() != len {
                return Err(Error::BoardSize("row sizes are not equal"));
            }
            board.extend(line.as_bytes());
        }

        Ok(Board { len, board })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn neighbors(&self, (x, y): (usize, usize)) -> Neighbors {
        Neighbors {
            x: x as isize,
            y: y as isize,
            current: 0,
            board: &self
        }
    }

    fn has_word(&self, word: &[u8]) -> bool {
        let mut visited = Visited::new(word.len(), self.len());

        for (k, b) in word.iter().cloned().enumerate() {
            for i in 0..self.len() {
                for j in 0..self.len() {
                    if k == 0 {
                        visited.visit((k, i, j));
                        continue;
                    }

                    if self[(i as isize, j as isize)] != b {
                        continue;
                    }

                    for (x, y) in self.neighbors((i, j)) {
                        if visited[(k - 1, x, y)] {
                            visited.visit((k, i, j));
                            if k == word.len() - 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    pub fn solve<'a, I: IntoIterator<Item=&'a str>>(&self, dictionary: I) -> usize {
        let words = dictionary.into_iter();
        let words: BTreeSet<_> = words.collect();
        words.iter().filter(|&w| {
            assert!(w.is_ascii(), format!("{:?}", w));
            self.has_word(w.as_bytes())
        }).count()
    }

    pub fn get(&self, (x, y): (isize, isize)) -> Option<&u8> {
        if x.is_negative() || x >= self.len() as isize || y.is_negative() || y >= self.len() as isize {
            None
        } else {
            let idx = self.len() as isize * x + y;
            self.board.get(idx as usize)
        }
    }
}

impl Index<(isize, isize)> for Board {
    type Output = u8;

    fn index(&self, idx: (isize, isize)) -> &u8 {
        self.get(idx).expect("index out of bounds!")
    }
}

struct Visited {
    word_len: usize,
    width: usize,
    visited: BitVec<u32>,
}

impl Visited {
    fn new(word_len: usize, width: usize) -> Visited{
        Visited {
            word_len,
            width,
            visited: BitVec::from_elem(word_len * width * width, false),
        }
    }

    fn idx(&self, (k, x, y): (usize, usize, usize)) -> Option<usize> {
        if k >= self.word_len || x >= self.width || y >= self.width {
            None
        } else {
            Some(self.width * self.width * y + self.width * x + k)
        }
    }

    fn visit(&mut self, idx: (usize, usize, usize)) {
        let idx = self.idx(idx).expect("index out of bounds");
        self.visited.set(idx, true)
    }
}

impl Index<(usize, usize, usize)> for Visited {
    type Output = bool;

    fn index(&self, idx: (usize, usize, usize)) -> &bool {
        &self.visited[self.idx(idx).expect("index out of bounds")]
    }
}

impl fmt::Debug for Visited {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Visited:\t")?; 
        for i in 0..self.word_len {
            write!(f, "{:?}:\t", i)?;
            for j in 0..self.width {
                for k in 0..self.width {
                    let idx = i + self.width * (j + self.width * k);
                    write!(f, "{:?}, ", self.visited[idx])?;
                }
                write!(f, "\n\t\t\t")?;
            }
            write!(f, "\n\t\t")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Neighbors<'a> {
    x: isize,
    y: isize,
    current: usize,
    board: &'a Board
}

impl<'a> Iterator for Neighbors<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= DIRECTIONS.len() {
            return None;
        }

        for &(x_off, y_off) in DIRECTIONS[self.current..].iter() {
            self.current += 1;
            let x = self.x + x_off;
            let y = self.y + y_off;
            if self.board.get((x, y)).is_some() {
                return Some((x as usize, y as usize))
            }
        }
        
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test::Bencher;

    const DICTIONARY: &str = include_str!("../test/dictionary");
    const BOARD: &str = "abcd\nefgh\nijkl\nmnop";
    const BOARD1: &str = include_str!("../test/board1");

    #[test]
    fn parse() {
        let board = Board::parse(BOARD).unwrap();
        assert_eq!(board.len(), 4);
        assert_eq!(board[(0, 0)], b'a');
        assert_eq!(board[(0, 3)], b'd');
        assert_eq!(board[(3, 3)], b'p');
        assert_eq!(board[(0, 1)], b'b');
        assert_eq!(board[(1, 0)], b'e');
    }

    #[test]
    fn neighbors_edge() {
        let board = Board::parse(BOARD).unwrap();
        let mut neighbors: Vec<_> = board.neighbors((0, 0)).collect();
        neighbors.sort();
        assert_eq!(neighbors, vec![(0, 1), (1, 0), (1, 1)]);

        let mut neighbors: Vec<_> = board.neighbors((3, 3)).collect();
        neighbors.sort();
        assert_eq!(neighbors, vec![(2, 2), (2, 3), (3, 2)]);
    }

    #[test]
    fn neighbors() {
        let board = Board::parse(BOARD).unwrap();
        let mut neighbors: Vec<_> = board.neighbors((1, 1)).collect();
        assert_eq!(board[(1, 1)], b'f');
        neighbors.sort();
        assert_eq!(neighbors, vec![
            (0, 0), (0, 1), (0, 2),
            (1, 0), (1, 2),
            (2, 0), (2, 1), (2, 2)
        ]);
    }

    #[test]
    fn has_word() {
        let board = Board::parse(BOARD).unwrap();
        assert!(board.has_word(b"abcd"));
        assert!(board.has_word(b"dcba"));
        assert!(board.has_word(b"afkp"));
        assert!(board.has_word(b"pkfa"));
        assert!(board.has_word(b"mjgd"));
        assert!(board.has_word(b"dgjm"));
        assert!(board.has_word(b"aeim"));
        assert!(board.has_word(b"miea"));
        assert!(board.has_word(b"aefb"));
        assert!(board.has_word(b"bfea"));

        assert!(!board.has_word(b"lies"));
        assert!(!board.has_word(b"mapb"));
    }

    #[bench]
    fn bench_dynamic_programming(b: &mut Bencher) {
        let board = Board::parse(BOARD1).unwrap();
        b.iter(|| {
            let dictionary = DICTIONARY.lines().filter(|l| l.len() > 3);
            board.solve(dictionary);
        });
    }
}

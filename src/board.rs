use std::ascii::AsciiExt;
use std::fmt;
use std::iter::{self, Iterator};
use std::ops::Index;

use rayon::prelude::*;

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

pub struct Board<'a> {
    board: Vec<&'a [u8]>,
}

impl<'a> fmt::Debug for Board<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Board:\t")?; 
        for i in 0..self.len() {
            write!(f, "\n\t{:?}", ::std::str::from_utf8(self.board[i]).unwrap())?;
        }
        Ok(())
    }
}

impl<'a> Board<'a> {
    pub fn parse(raw: &str) -> Result<Board, Error> {
        debug_assert!(raw.is_ascii());
        let board: Vec<_> = raw.lines().map(|l| l.as_bytes()).collect();
        if board.iter().any(|l| l.len() != board.len()) {
            return Err(Error::BoardSize("unequal row and column sizes"));
        }

        Ok(Board { board })
    }

    pub fn len(&self) -> usize {
        self.board.len()
    }

    fn neighbors(&self, (x, y): (usize, usize)) -> Neighbors {
        Neighbors {
            x: x as isize,
            y: y as isize,
            current: 0,
            board: &self
        }
    }

    fn has_word(&self, visited: &mut Visited, word: &[u8]) -> bool {
        visited.reset(word.len(), self.len());
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

    pub fn solve_single_threaded<R: AsRef<str>>(&self, words: R) -> usize {
        let mut visited = Visited::new();

        words.as_ref()
            .lines()
            .filter(|&w| w.len() > 3 && self.has_word(&mut visited, w.as_bytes()))
            .count()
    }

    pub fn solve_rayon<R: AsRef<str>>(&self, words: R) -> usize {
        words.as_ref()
            .par_lines()
            .filter(|&w| w.len() > 3 && self.has_word(&mut Visited::new(), w.as_bytes()))
            .count()
    }

    pub fn solve_trie<R: AsRef<str>>(&self, words: R) -> usize {
        let mut visited = Visited::new();
        let arena = Arena::new();
        let trie = TrieNode::root(&arena);

        for word in words.as_ref().lines() {
            trie.insert(word.as_bytes(), &arena);
        }

        let mut count = 0;
        let mut words_searched = 0;
        let mut to_search = Vec::with_capacity(1024);
        to_search.push(trie);

        while let Some(next) = to_search.pop() {
            if next.word_end && next.word.len() > 3 {
                words_searched +=1;
                if self.has_word(&mut visited, next.word) {
                    count +=1;
                } else {
                    continue;
                }
            }

            for root in next.roots.iter() {
                let r = root.take();
                if let Some(r) = r {
                    to_search.push(r);
                }
                root.set(r);
            }
        }
        println!("trie words searched: {}", words_searched);
        count
    }

    pub fn get(&self, (x, y): (isize, isize)) -> Option<&u8> {
        if x.is_negative() || x >= self.len() as isize || y.is_negative() || y >= self.len() as isize {
            None
        } else {
            self.board.get(x as usize).and_then(|r| r.get(y as usize))
        }
    }
}

impl<'a> Index<(isize, isize)> for Board<'a> {
    type Output = u8;

    fn index(&self, idx: (isize, isize)) -> &u8 {
        self.get(idx).expect("index out of bounds!")
    }
}

struct Visited {
    word_len: usize,
    width: usize,
    visited: Vec<bool>,
}

impl Visited {
    fn new() -> Visited{
        Visited {
            word_len: 0,
            width: 0,
            // 16 (longest word in included corpus) * 6 (width) * 6 (width) = 576
            // round up to the nearest power of two and bob's your uncle
            visited: Vec::with_capacity(1024),
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
        self.visited[idx] = true;
    }

    fn reset(&mut self, word_len: usize, width: usize) {
        self.word_len = word_len;
        self.width = width;
        let new_len = word_len * width * width;
        let old_len = self.visited.len();

        if new_len > old_len {
            self.visited.reserve(new_len - old_len);
        }
        self.visited.truncate(0);
        // this would be faster if we had safe fill on stable rust
        // or llvm did a better job of using memset
        self.visited.extend(iter::repeat(false).take(new_len));
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
struct Neighbors<'a, 'b: 'a> {
    x: isize,
    y: isize,
    current: usize,
    board: &'a Board<'b>,
}

impl<'a, 'b> Iterator for Neighbors<'a, 'b> {
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
    const BOARD: &str = "abcd\nefgh\nijkl\nmnop";

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
        fn has_word(board: &Board, word: &[u8]) -> bool {
            board.has_word(&mut Visited::new(), word)
        };
        assert!(has_word(&board, b"abcd"));
        assert!(has_word(&board, b"dcba"));
        assert!(has_word(&board, b"afkp"));
        assert!(has_word(&board, b"pkfa"));
        assert!(has_word(&board, b"mjgd"));
        assert!(has_word(&board, b"dgjm"));
        assert!(has_word(&board, b"aeim"));
        assert!(has_word(&board, b"miea"));
        assert!(has_word(&board, b"aefb"));
        assert!(has_word(&board, b"bfea"));

        assert!(!has_word(&board, b"lies"));
        assert!(!has_word(&board, b"mapb"));
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    use super::*;
    use test::Bencher;

    const DICTIONARY: &str = include_str!("../test/dictionary");
    const BOARD1: &str = include_str!("../test/board1");

    #[bench]
    fn bench_single_threaded(b: &mut Bencher) {
        let board = Board::parse(BOARD1).unwrap();
        b.iter(|| {
            board.solve_single_threaded(DICTIONARY);
        });
    }

    #[bench]
    fn bench_rayon(b: &mut Bencher) {
        let board = Board::parse(BOARD1).unwrap();
        b.iter(|| {
            board.solve_rayon(DICTIONARY);
        });
    }
}

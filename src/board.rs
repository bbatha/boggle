use std::ascii::AsciiExt;
use std::fmt;
use std::iter::Iterator;
use std::ops::Index;
use std::str;

use typed_arena::Arena;

use error::Error;
use trie::TrieNode;
use multivec::{Vec2, Vec3};

pub struct Board<'word> {
    board: Vec<&'word [u8]>,
    letters: [bool; 26],
}

impl<'word> fmt::Debug for Board<'word> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Board:\t")?; 
        for row in self.board.iter() {
            write!(f, "\n\t{:?}", str::from_utf8(row).expect("board is ascii"))?
        }
        Ok(())
    }
}

impl<'word> Board<'word> {
    pub fn parse(raw: &str) -> Result<Board, Error> {
        assert!(raw.is_ascii());
        let board: Vec<_> = raw.lines().map(|l| l.as_bytes()).collect();
        if board.iter().any(|l| l.len() != board.len()) {
            return Err(Error::BoardSize("unequal row and column sizes"));
        }

        let mut letters = [false; 26];
        for c in board.iter().flat_map(|r| r.iter().cloned()) {
            letters[(c - b'a') as usize] = true;
        }
        Ok(Board { board, letters })
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

    fn contains_letters(&self, word: &[u8]) -> bool {
        word.iter().all(|&w| self.letters[(w - b'a') as usize])
    }

    // checks to see if basic conditions for the existance of a word are met
    // e.g.
    // are all the letters in the board
    // is the word long enough
    // are the letters of the word found in adjacent to each other
    // you still need to check to see if the word reuses a letter after calling this method
    fn has_word(&self, word: &[u8]) -> bool {
        let mut adjacencies = Vec3::fill(word.len(), self.len(), self.len(), false);
        for (k, &b) in word.iter().enumerate() {
            for i in 0..self.len() {
                for j in 0..self.len() {
                    if b != self[(i, j)] {
                        continue;
                    }

                    if k == 0 {
                        adjacencies[(k, i, j)] = true;
                        continue;
                    }

                    for (x, y) in self.neighbors((i, j)) {
                        if adjacencies[(k - 1, x, y)] {
                            adjacencies[(k, i, j)] = true;
                            if word.len() - 1 == k {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    pub fn get(&self, (x, y): (isize, isize)) -> Option<&u8> {
        if x.is_negative() || x >= self.len() as isize || y.is_negative() || y >= self.len() as isize {
            None
        } else {
            self.board.get(x as usize).and_then(|r| r.get(y as usize))
        }
    }

    pub fn solve_single_threaded<'a>(&self, words: &'a str) -> Vec<&'a str> {
        #[derive(Debug)]
        struct DfsItem<'word> {
            visited: Vec2<bool>,
            x: usize,
            y: usize,
            word: &'word str,
        }

        let mut solutions = Vec::new();
        let mut stack = Vec::with_capacity(4098);
        for word in words.lines() {
            if word.as_bytes().len() < 3 || !self.contains_letters(word.as_bytes()) || !self.has_word(word.as_bytes()) {
                continue;
            }

            stack.truncate(0);
            'found: for i in 0..self.len() {
                for j in 0..self.len() {
                    let visited = Vec2::fill(self.len(), self.len(), false);
                    stack.push(DfsItem { x: i, y: j, visited, word: &word[0..1] });

                    while let Some(mut curr) = stack.pop() {
                        if self[(curr.x, curr.y)] != *curr.word.as_bytes().last().unwrap() {
                            continue;
                        }

                        if curr.word.len() == word.len() {
                            solutions.push(word);
                            break 'found;
                        }

                        curr.visited[(curr.x, curr.y)] = true;
                        for (x, y) in self.neighbors((curr.x, curr.y)) {
                            if !curr.visited[(x, y)] {
                                stack.push(DfsItem { x, y, visited: curr.visited.clone(), word: &word[0..curr.word.len() + 1] });
                            }
                        }
                    }
                }
            }
        }

        solutions
    }

    pub fn solve_trie<'a>(&self, words: &'a str) -> Vec<&'a str> {
        let arena = Arena::new();
        let trie = TrieNode::root(&arena);

        for word in words.lines() {
            if word.len() >= 3 && self.contains_letters(word.as_bytes()) {
                trie.insert(word.as_bytes(), &arena);
            }
        }

        #[derive(Debug)]
        struct DfsItem<'trie, 'word: 'trie> {
            visited: Vec2<bool>,
            x: usize,
            y: usize,
            trie: &'trie TrieNode<'trie, 'word>,
        }

        let mut stack = Vec::with_capacity(4098);
        let mut solutions = Vec::new();
        for i in 0..self.len() {
            for j in 0..self.len() {
                stack.truncate(0);
                let visited = Vec2::fill(self.len(), self.len(), false);
                stack.push(DfsItem { x: i, y: j, trie, visited });

                while let Some(mut curr) = stack.pop() {
                    curr.visited[(curr.x, curr.y)] = true;

                    for (x, y) in self.neighbors((curr.x, curr.y)) {
                        let next = curr.trie.get(self[(x, y)]);
                        if let Some(next) = next {
                            if !curr.visited[(x, y)] {
                                stack.push(DfsItem { trie: next, x, y, visited: curr.visited.clone() });
                            }
                        }
                    }

                    if !curr.trie.seen.replace(true) && curr.trie.word_end {
                        solutions.push(unsafe { str::from_utf8_unchecked(curr.trie.word )});
                    }
                }
            }
        }

        solutions
    }
}

impl<'word> Index<(usize, usize)> for Board<'word> {
    type Output = u8;

    fn index(&self, (x, y): (usize, usize)) -> &u8 {
        self.get((x as isize, y as isize)).expect("index out of bounds!")
    }
}

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

#[derive(Debug)]
struct Neighbors<'board, 'word: 'board> {
    x: isize,
    y: isize,
    current: usize,
    board: &'board Board<'word>,
}

impl<'board, 'word> Iterator for Neighbors<'board, 'word> {
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

    const DICTIONARY: &str = include_str!("../test/dictionary");
    const BOARD1: &str = include_str!("../test/board1");

    #[test]
    fn single_threaded() {
        let board = Board::parse(BOARD1).unwrap();
        assert_eq!(board.solve_single_threaded(DICTIONARY).len(), 126);
    }

    #[test]
    fn trie() {
        let board = Board::parse(BOARD1).unwrap();
        assert_eq!(board.solve_trie(DICTIONARY).len(), 126);
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
    fn bench_trie(b: &mut Bencher) {
        let board = Board::parse(BOARD1).unwrap();
        b.iter(|| {
            board.solve_trie(DICTIONARY);
        });
    }
}
use std::cell::Cell;
use std::ops::Index;

use typed_arena::Arena;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct TrieNode<'trie, 'word: 'trie> {
    pub word: &'word [u8],
    pub word_end: bool,
    pub seen: Cell<bool>,
    pub roots: [Cell<Option<&'trie TrieNode<'trie, 'word>>>; 26]
}

impl<'trie, 'word> TrieNode<'trie, 'word> {
    pub fn root(arena: &'trie Arena<TrieNode<'trie, 'word>>) -> &'trie TrieNode<'trie, 'word> {
        TrieNode::new(false, &[], arena)
    }

    pub fn new(word_end: bool, word: &'word [u8], arena: &'trie Arena<TrieNode<'trie, 'word>>) -> &'trie TrieNode<'trie, 'word> {
        arena.alloc(TrieNode {
            word_end,
            word,
            seen: Cell::new(false),
            roots: [
                Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None),
                Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None),
                Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None),
                Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None),
                Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None), Cell::new(None),
                Cell::new(None),
            ]
        })
    }

    pub fn insert(&'trie self, word: &'word [u8], arena: &'trie Arena<TrieNode<'trie, 'word>>) {
        let mut last = self;
        for l in 0..word.len() {
            let c = word[l];
            let root = last[c].take();
            let child = if let Some(root) = root {
                root
            } else {
                TrieNode::new(l == word.len() - 1, &word[..l+1], arena)
            };
            last[c].set(Some(child));
            last = child;
        }
        
    }

    pub fn contains(&self, word: &[u8]) -> bool {
        let mut last = self;
        for &c in word {
            if let Some(root) = last.get(c) {
                last = root;
            } else {
                return false;
            }
        }
        true
    }

    pub fn get(&self, c: u8) -> Option<&'trie TrieNode<'trie, 'word>> {
        if c < b'a' || c > b'z' {
            None
        } else {
            let idx = (c - b'a') as usize;
            let child = self.roots[idx].take();
            self.roots[idx].set(child);
            child
        }
    }
}

impl<'trie, 'word> Index<u8> for TrieNode<'trie, 'word> {
    type Output = Cell<Option<&'trie TrieNode<'trie, 'word>>>;

    fn index(&self, c: u8) -> &Self::Output {
        assert!(c >= b'a');
        assert!(c <= b'z');
        let idx = (c - b'a') as usize;
        &self.roots[idx]
    }
}
#[test]
fn smoke() {
    let arena = Arena::new();
    let trie = TrieNode::root(&arena);
    let words: &[&[u8]] = &[b"test", b"foo", b"bar", b"baz"];

    for word in words {
        trie.insert(word, &arena);
    }

    assert!(trie.contains(b"test"));
    assert!(trie.contains(b"foo"));
    assert!(trie.contains(b"bar"));
    assert!(trie.contains(b"baz"));
    assert!(!trie.contains(b"dne"));
}
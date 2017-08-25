use std::ops::{Index, IndexMut};
use std::fmt::{self, Debug};
use std::iter;

use smallvec::SmallVec;

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct Vec3<T> {
    height: usize,
    depth: usize,
    width: usize,
    data: SmallVec<[T; 2048]>, // 20 characters * 8 x 8 board
}

impl<T> Vec3<T> {
    fn idx(&self, (x, y, z): (usize, usize, usize)) -> Option<usize> {
        if z >= self.depth || x >= self.width || y >= self.height {
            None
        } else {
            Some(self.height * self.width * z + self.width * y + x)
        }
    }

    pub fn fill(width: usize, height: usize, depth: usize, value: T) -> Vec3<T>
        where T: Clone
    {
        let data = iter::repeat(value).take(width * height * depth).collect(); 
        Vec3 {
            width,
            height,
            depth,
            data,
        }
    }
}

impl<T> Index<(usize, usize, usize)> for Vec3<T> {
    type Output = T;

    fn index(&self, idx: (usize, usize, usize)) -> &T {
        &self.data[self.idx(idx).expect("index out of bounds")]
    }
}

impl<T> IndexMut<(usize, usize, usize)> for Vec3<T> {
    fn index_mut(&mut self, idx: (usize, usize, usize)) -> &mut T {
        let idx = self.idx(idx).expect("index out of bounds");
        &mut self.data[idx]
    }
}

impl<T: Debug> Debug for Vec3<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3:\t")?; 
        for i in 0..self.width {
            write!(f, "{:?}:\t", i)?;
            for j in 0..self.height {
                for k in 0..self.depth {
                    let idx = self.idx((i, j, k)).unwrap();
                    write!(f, "{:?}, ", self.data[idx])?;
                }
                write!(f, "\n\t\t")?;
            }
            write!(f, "\n\t")?;
        }
        Ok(())
    }
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct Vec2<T> {
    height: usize,
    width: usize,
    data: SmallVec<[T; 64]>, // 8 x 8 board
}

impl<T> Vec2<T> {
    fn idx(&self, (x, y): (usize, usize)) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(x + y * self.width)
        }
    }

    pub fn fill(width: usize, height: usize, value: T) -> Vec2<T>
        where T: Clone
    {

        let data = iter::repeat(value).take(width * height).collect();
        Vec2 {
            width,
            height,
            data,
        }
    }
}

impl<T> Index<(usize, usize)> for Vec2<T> {
    type Output = T;

    fn index(&self, idx: (usize, usize)) -> &T {
        &self.data[self.idx(idx).expect("index out of bounds")]
    }
}

impl<T> IndexMut<(usize, usize)> for Vec2<T> {
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut T {
        let idx = self.idx(idx).expect("index out of bounds");
        &mut self.data[idx]
    }
}

impl<T: Debug> Debug for Vec2<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3:\t")?; 
        for i in 0..self.width {
            write!(f, "{:?}:\t", i)?;
            for j in 0..self.height {
                let idx = self.idx((i, j)).unwrap();
                write!(f, "{:?}, ", self.data[idx])?;
            }
            write!(f, "\n\t")?;
        }
        Ok(())
    }
}

#[test]
fn smoke() {
    let mut v = Vec3::fill(3, 4, 4, false);
    {
        v[(1, 2, 0)] = true;
    }
    println!("{:?}", v);
    assert!(v[(1, 2, 0)]);
}
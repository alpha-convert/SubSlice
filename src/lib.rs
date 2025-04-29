use std::{cell::Cell, ops::{Deref, DerefMut}};

#[derive(Debug, Clone, Copy)]
enum BorrowState {
    Not,
    Shared(usize),
    Mutable
}

pub struct SubSlice<'a,T> {
    data : &'a [T],
    borrows : Cell<&'a mut [BorrowState]>
}

impl<'a,T> SubSlice<'a,T> {
    pub fn new(raw : &'a mut [T]) -> Self {
        let len = raw.len();
        let init_borrows = Cell::new(vec![BorrowState::Not;len].leak());
        SubSlice { data: raw, borrows: init_borrows }
    }

    pub fn sub(&'a self, start : usize, end : usize) -> Sub<'a,T> {
        assert!(start <= end && end < self.data.len());
        let borrows = self.borrows.take();
        borrows[start..end].iter_mut().enumerate().for_each(|(i,b)| {
            match b {
                BorrowState::Not => *b = BorrowState::Shared(1),
                BorrowState::Shared(n) => { *n += 1 },
                BorrowState::Mutable => panic!("Cannot shared borrow already mutably borrowed index {}",i),
            }
        });
        self.borrows.set(borrows);
        Sub {
            parent: &self,
            start,
            end,
            data: &self.data[start..end],
        }
    }

    pub fn sub_mut(&'a self, start : usize, end : usize) -> SubMut<'a,T> {
        assert!(start <= end && end < self.data.len());

        let borrows = self.borrows.take();
        borrows[start..end].iter_mut().enumerate().for_each(|(i,b)| {
            match b {
                BorrowState::Not => *b = BorrowState::Mutable,
                BorrowState::Shared(_) => panic!("Cannot mutably borrow already shared-borrowed index {}", i),
                BorrowState::Mutable => panic!("Cannot mutably borrow already mutably borrowed index {}",i),
            }
        });
        self.borrows.set(borrows);


        let p = self.data.as_ptr() as *mut T;
        let d = unsafe {
            let d_start = p.add(start);
            std::slice::from_raw_parts_mut(d_start, end - start)
        };

        SubMut {
            parent: self,
            start,
            end,
            data: d,
        }

    }

    fn relinquish(&self, start : usize, end : usize) {
        let borrows = self.borrows.take();
        for (i,b) in &mut borrows[start..end].iter_mut().enumerate() {
            match b {
                BorrowState::Not => panic!("Cannot relinquish unborrowed index {}",i),
                BorrowState::Shared(0) => { *b = BorrowState::Not },
                BorrowState::Shared(n) => { *n -= 1 },
                BorrowState::Mutable => { *b = BorrowState::Not },
            }
        }
        self.borrows.set(borrows);
    }
}

pub struct Sub<'a,T> {
    parent : &'a SubSlice<'a,T>,
    start : usize,
    end : usize,
    data : &'a [T]
}

impl<'a,T> Deref for Sub<'a,T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a,T> AsRef<[T]> for Sub<'a,T> {
    fn as_ref(&self) -> &[T] {
        self.data
    }
}

impl<'a,T> Drop for Sub<'a,T> {
    fn drop(&mut self) {
        self.parent.relinquish(self.start, self.end);
    }
}

pub struct SubMut<'a,T> {
    parent : &'a SubSlice<'a,T>,
    start : usize,
    end : usize,
    data : &'a mut [T]
}

impl<'a,T> Deref for SubMut<'a,T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a,T> AsRef<[T]> for SubMut<'a,T> {
    fn as_ref(&self) -> &[T] {
        self.data
    }
}

impl<'a,T> AsMut<[T]> for SubMut<'a,T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.data
    }
}

impl <'a,T> DerefMut for SubMut<'a,T>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a,T> Drop for SubMut<'a,T> {
    fn drop(&mut self) {
        self.parent.relinquish(self.start, self.end);
    }
}

#[test]
fn foo(){
    let mut xs = [1,2,3,4,5];

    let sl = SubSlice::new(&mut xs);

    let a = sl.sub(0, 1);
    let b = sl.sub(1, 2);
    let c = sl.sub(1, 2);
    let d = sl.sub_mut(2, 4);
    let e = sl.sub(0, 1);
    let f = sl.sub(1, 2);
}
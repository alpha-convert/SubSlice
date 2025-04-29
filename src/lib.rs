use std::{cell::Cell, ops::{Deref, DerefMut}, sync::{Arc, Mutex}};

use borrowstate::BorrowState;
use borrowtracker::BorrowTracker;
use vectracker::VecTracker;

mod borrowstate;
mod borrowtracker;
mod vectracker;


pub struct SubSlice<'a,T> {
    data : &'a [T],
    borrows : Mutex<VecTracker>
}

impl<'a,T> SubSlice<'a,T> {
    pub fn new(raw : &'a mut [T]) -> Self {
        let len = raw.len();
        let init_borrows = Mutex::new(VecTracker::new(len));
        SubSlice { data: raw, borrows: init_borrows }
    }

    pub fn sub(&'a self, start : usize, end : usize) -> Sub<'a,T> {
        assert!(start <= end && end < self.data.len());
        let mut borrows = self.borrows.lock().unwrap();
        borrows.add_shr(start, end);
        Sub {
            parent: &self,
            start,
            end,
            data: &self.data[start..end],
        }
    }

    pub fn sub_mut(&'a self, start : usize, end : usize) -> SubMut<'a,T> {
        assert!(start <= end && end < self.data.len());

        let mut borrows = self.borrows.lock().unwrap();
        borrows.add_mut(start, end);

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
        let mut borrows = self.parent.borrows.lock().unwrap();
        borrows.rm_shr(self.start, self.end);
        // self.parent.borrows.set(borrows);
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
        let mut borrows = self.parent.borrows.lock().unwrap();
        borrows.rm_mut(self.start, self.end);
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
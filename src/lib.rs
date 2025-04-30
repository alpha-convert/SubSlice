#![feature(btree_cursors)]
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use borrowstate::BorrowState;
use borrowtracker::BorrowTracker;
use btreetracker::BTreeTracker;

pub mod borrowstate;
pub mod borrowtracker;
pub mod btreetracker;
pub mod vectracker;

pub struct SubSlice<'a, T, B = BTreeTracker> {
    data: &'a mut [T],
    borrows: Mutex<B>,
}

impl<'a, T, B> SubSlice<'a, T, B>
where
    B: BorrowTracker,
{
    pub fn new(raw: &'a mut [T]) -> Self {
        let len = raw.len();
        let init_borrows = Mutex::new(B::new(len));
        SubSlice {
            data: raw,
            borrows: init_borrows,
        }
    }

    pub fn sub(&'a self, start: usize, end: usize) -> Sub<'a, T, B> {
        assert!(start < end && end < self.data.len());
        let mut borrows = self.borrows.lock().unwrap();
        borrows.add_shr(start, end);
        Sub {
            parent: &self,
            start,
            end,
            data: &self.data[start..end],
        }
    }

    pub fn sub_mut(&'a self, start: usize, end: usize) -> SubMut<'a, T, B> {
        assert!(start < end && end < self.data.len());

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

pub struct Sub<'a, T, B: BorrowTracker> {
    parent: &'a SubSlice<'a, T, B>,
    start: usize,
    end: usize,
    data: &'a [T],
}

impl<'a, T, B: BorrowTracker> Deref for Sub<'a, T, B> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> AsRef<[T]> for Sub<'a, T, B> {
    fn as_ref(&self) -> &[T] {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> Drop for Sub<'a, T, B> {
    fn drop(&mut self) {
        let mut borrows = self.parent.borrows.lock().unwrap();
        borrows.rm_shr(self.start, self.end);
        // self.parent.borrows.set(borrows);
    }
}

pub struct SubMut<'a, T, B: BorrowTracker> {
    parent: &'a SubSlice<'a, T, B>,
    start: usize,
    end: usize,
    data: &'a mut [T],
}

impl<'a, T, B: BorrowTracker> Deref for SubMut<'a, T, B> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> AsRef<[T]> for SubMut<'a, T, B> {
    fn as_ref(&self) -> &[T] {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> AsMut<[T]> for SubMut<'a, T, B> {
    fn as_mut(&mut self) -> &mut [T] {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> DerefMut for SubMut<'a, T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T, B: BorrowTracker> Drop for SubMut<'a, T, B> {
    fn drop(&mut self) {
        let mut borrows = self.parent.borrows.lock().unwrap();
        borrows.rm_mut(self.start, self.end);
    }
}

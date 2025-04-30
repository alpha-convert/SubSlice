use std::{collections::BTreeMap, ops::Bound};

use crate::{borrowstate::BorrowState, borrowtracker::BorrowTracker};

#[derive(Debug)]
pub struct BTreeTracker {
    len: usize, // length of the underlying
    borrows: BTreeMap<usize, BorrowState>,
}

impl BorrowTracker for BTreeTracker {
    fn new(len: usize) -> Self {
        let mut b = BTreeMap::new();
        b.insert(0, BorrowState::Not);
        b.insert(len, BorrowState::Not);
        BTreeTracker {
            len: len,
            borrows: b,
        }
    }

    fn add_shr(&mut self, start: usize, end: usize) {
        self.borrows
            .entry(start)
            .and_modify(|b| b.add_shr())
            .or_insert(BorrowState::Shared(1));

        let mut cur = self.borrows.lower_bound_mut(Bound::Excluded(&start));
        let mut end_state = BorrowState::Not;
        while let Some((i, b)) = cur.next() {
            if *i >= end {
                break;
            } else {
                end_state = *b;
                b.add_shr();
            }
        }
        self.borrows.entry(end).or_insert(end_state);
    }

    fn add_mut(&mut self, start: usize, end: usize) {
        self.borrows
            .entry(start)
            .and_modify(|b| b.add_mut())
            .or_insert(BorrowState::Mutable);

        let mut cur = self.borrows.lower_bound_mut(Bound::Excluded(&start));
        while let Some((i, b)) = cur.next() {
            if *i >= end {
                break;
            } else {
                b.add_mut();
            }
        }
        self.borrows.entry(end).or_insert(BorrowState::Not);
    }

    fn rm_shr(&mut self, start: usize, end: usize) {
        let mut cur = self.borrows.lower_bound_mut(Bound::Included(&start));
        while let Some((i, b)) = cur.next() {
            if *i >= end {
                break;
            } else {
                b.rm_shr();
            }
        }
    }

    //NOTE: At the moment, this doesn't garbage collect the tree, removing unneeded (intermediate) `Not`s as we go...
    //A Not is unneeded if its predecessor is also a Not
    //More generally, a node is unneeded if it's identical to its predecessor.
    fn rm_mut(&mut self, start: usize, end: usize) {
        let mut cur = self.borrows.lower_bound_mut(Bound::Included(&start));
        while let Some((i, b)) = cur.next() {
            if *i >= end {
                break;
            } else {
                b.rm_mut();
            }
        }
    }

    fn into_state(&self) -> Vec<BorrowState> {
        let mut v = vec![BorrowState::Not; self.len];
        let mut cur = BorrowState::Not;
        for (i, rb) in v.iter_mut().enumerate() {
            match self.borrows.get(&i) {
                None => {}
                Some(b) => {
                    cur = *b;
                }
            };
            *rb = cur;
        }
        v
    }
}

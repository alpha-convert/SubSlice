use crate::{borrowstate::BorrowState, borrowtracker::BorrowTracker};

pub struct VecTracker {
    borrows : Vec<BorrowState>
}

impl BorrowTracker for VecTracker {
    fn add_shr(&mut self, start : usize, end : usize) {
        for b in self.borrows.as_mut_slice()[start..end].iter_mut() {
            b.add_shr();
        }
    }

    fn add_mut(&mut self, start : usize, end : usize) {
        for b in self.borrows.as_mut_slice()[start..end].iter_mut() {
            b.add_mut();
        }
    }
    
    fn rm_shr(&mut self, start : usize, end : usize) {
        for b in self.borrows.as_mut_slice()[start..end].iter_mut() {
            b.rm_shr();
        }
    }
    
    fn rm_mut(&mut self, start : usize, end : usize) {
        for b in self.borrows.as_mut_slice()[start..end].iter_mut() {
            b.rm_mut();
        }
    }
    
    fn into_state(self) -> Vec<BorrowState> {
        self.borrows
    }
    
    fn new(len : usize) -> Self {
        VecTracker { borrows: vec![BorrowState::Not;len] }
    }
}
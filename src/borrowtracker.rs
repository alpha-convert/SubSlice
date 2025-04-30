use crate::borrowstate::BorrowState;

pub trait BorrowTracker {
    fn new(len : usize) -> Self;
    fn add_shr(&mut self, start : usize, end : usize);
    fn add_mut(&mut self, start : usize, end : usize);
    fn rm_shr(&mut self, start : usize, end : usize);
    fn rm_mut(&mut self, start : usize, end : usize);

    fn into_state(self) -> Vec<BorrowState>;
}
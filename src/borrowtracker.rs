pub trait BorrowTracker : Default {
    fn add_shr(&mut self, start : usize, end : usize);
    fn add_mut(&mut self, start : usize, end : usize);
    fn rm_shr(&mut self, start : usize, end : usize);
    fn rm_mut(&mut self, start : usize, end : usize);
}
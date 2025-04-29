pub struct BorrowTrackerReference<'a,T> {
    borrows : &'a mut [BorrowState]
}
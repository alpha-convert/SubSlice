#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BorrowState {
    Not,
    Shared(usize),
    Mutable
}

impl BorrowState {
    pub fn add_shr(&mut self) {
        match self {
            BorrowState::Not => {*self = Self::Shared(1)},
            BorrowState::Shared(n) => { *n += 1 },
            BorrowState::Mutable => panic!(""),
        }
    }

    pub fn rm_shr(&mut self) {
        match self {
            BorrowState::Not => { panic!("") },
            BorrowState::Shared(1) => { *self = BorrowState::Not },
            BorrowState::Shared(n) => { *n -= 1 },
            BorrowState::Mutable => panic!(""),
        }
    }

    pub fn add_mut(&mut self) {
        match self {
            BorrowState::Not => { *self = Self::Mutable },
            BorrowState::Shared(n) => { panic!("") },
            BorrowState::Mutable => panic!(""),
        }
    }

    pub fn rm_mut(&mut self) {
        match self {
            BorrowState::Not => { panic!("") },
            BorrowState::Shared(_) => { panic!("") },
            BorrowState::Mutable => { *self = BorrowState::Not },
        }
    }
}
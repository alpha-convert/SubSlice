use std::{ops::Bound, time::Duration};

use subslice::{borrowtracker::BorrowTracker, btreetracker::BTreeTracker, vectracker::VecTracker};
use itertools::{Itertools, Positions};


#[derive(Debug,Clone,Copy)]
enum BorrowAction {
    AddShared(usize,usize),
    AddMutable(usize,usize),
    RmShared(usize,usize),
    RmMutable(usize,usize),
}

struct ActionGenerator {
    len : usize,
    num_actions : usize
}

impl ActionGenerator {
    fn generate_action<D: bolero::Driver>(&self,driver: &mut D, active_shared: &mut [u32], active_mutable: &mut [bool]) -> Option<BorrowAction> {

        let mut possible_actions = Vec::new();
        for i in (0..self.len) {
            for j in ((i+1)..self.len) {
                let any_mutable = active_mutable[i..j].iter().any(|b| {*b == true});
                let all_mutable = active_mutable[i..j].iter().all(|b| {*b == true});
                let any_shared = active_shared[i..j].iter().any(|b| {*b > 0});
                let all_shared = active_shared[i..j].iter().all(|b| {*b > 0});

                if !any_mutable && !any_shared{
                    possible_actions.push(BorrowAction::AddMutable(i,j))
                }

                if !any_mutable {
                    possible_actions.push(BorrowAction::AddShared(i, j));
                }

                if all_mutable {
                    possible_actions.push(BorrowAction::RmMutable(i, j))
                }

                if all_shared {
                    possible_actions.push(BorrowAction::RmMutable(i, j));
                }

            }
        }

        let i = driver.gen_usize(Bound::Included(&0), Bound::Excluded(&possible_actions.len()))?;
        let act = possible_actions[i];
        match act {
            BorrowAction::AddShared(i, j) => {
                active_shared[i..j].iter_mut().for_each(|b| {*b += 1});
            },
            BorrowAction::AddMutable(i, j) => {
                active_mutable[i..j].iter_mut().for_each(|b| {*b = true});
            },
            BorrowAction::RmShared(i, j) => {
                active_shared[i..j].iter_mut().for_each(|b| {*b -= 1});
            },
            BorrowAction::RmMutable(i, j) => {
                active_mutable[i..j].iter_mut().for_each(|b| {*b = false});
            },
        };
        Some(act)

    }

}


impl bolero::generator::ValueGenerator for ActionGenerator {
    type Output = Vec<BorrowAction>;

    fn generate<D: bolero::Driver>(&self, driver: &mut D) -> Option<Self::Output> {
        let mut actions = Vec::new();
        
        // Track active borrows to ensure validity
        let mut active_shared = vec![0; self.len];
        let mut active_mutable = vec![false; self.len];
        
        for _ in 0..self.num_actions {
            let ma = self.generate_action(driver, &mut active_shared, &mut active_mutable);
            match ma {
                Some(a) => actions.push(a),
                None => {}
            }
        }
        
        Some(actions)
    }
}




#[test]
fn difftest(){
    let len = 5;
    let num_actions = 4;
    let g = ActionGenerator { len, num_actions };
    bolero::check!().with_generator(g).with_iterations(100).for_each(|actions| {
        let mut reference = VecTracker::new(len);
        let mut btree = BTreeTracker::new(len);
        for action in actions {
            match action {
                BorrowAction::AddShared(start, end) => {
                    reference.add_shr(*start, *end);
                    btree.add_shr(*start, *end);
                },
                BorrowAction::AddMutable(start, end) => {
                    reference.add_mut(*start, *end);
                    btree.add_mut(*start, *end);
                },
                BorrowAction::RmShared(start, end) => {
                    reference.rm_shr(*start, *end);
                    btree.rm_shr(*start, *end);
                },
                BorrowAction::RmMutable(start, end) => {
                    reference.rm_mut(*start, *end);
                    btree.rm_mut(*start, *end);
                },
            }
        }

        let refrresult = reference.into_state();
        let btreeresult = btree.into_state();

        assert!(refrresult == btreeresult)
    })

}
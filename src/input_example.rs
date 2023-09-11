/// This would be the Ohua input for a test case
use crate::funs::*;

pub fn test(i:i32) -> i32 {
    let mut s: State = State::new_state(i);
    let stream: Vec<i32> = iter_i32();
    for e in stream {
        let e1: i32 = e;
        s.gs(e1);
    }
    s.gs(5)
}
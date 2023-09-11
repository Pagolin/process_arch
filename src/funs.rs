#[derive(Copy, Clone, Debug)]
pub struct State{
    val:i32,
}


impl State {
    pub fn new(val:i32) -> Self {
        State{val}
    }

    pub fn new_state(val:i32) -> Self {
        State{val}
    }

    pub fn do_stuff(&self, i:i32) -> i32 {
        23
    }

    pub fn io(&self) {
        println!("LOOOP gnihihi\n")
    }

    pub fn gs(&mut self, num:i32) -> i32 {
        self.val += num;
        return self.val
    }
}

pub fn a(arg: u32) -> u32 {
    arg + 3
}

pub fn b(arg: u32) -> u32 {
    arg * 2
}

pub fn c() -> u32 {
    4
}

pub fn h(i:i32) -> i32 {
    let the_answer = if i==23 {i} else {i+1};
    the_answer
}

pub fn h2(i:i32) -> () {
    let the_answer = if i==23 {i+19} else {42};
    println!("Calls h2 with {}", i)
}

pub fn h3(i:i32, j:i32) -> () {
    let the_answer = if i==23 {j} else {42};
    println!("Calls h2 with {}", i)
}

pub fn check(i:i32) -> bool {
    i < 23
}

pub fn host_id(i:bool) -> (bool, bool) {
    (i,i)
}

pub fn iter_i32() -> Vec<i32> {
    (1..11).collect()
}

pub type ThisIsActuallyUnit = ();

pub fn make_unit() -> ThisIsActuallyUnit {
    ()
}

pub fn f_s(state:State, i:i32) -> i32 {
    println!("State: {:?}, i: {:?}", state.val, i);
    state.val - i
}

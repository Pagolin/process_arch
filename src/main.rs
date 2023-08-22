// Currently we use pipes, implying memory copies. For less overhead but also less protection we could use 
// shared memory queues loosing isolation but gaining performance.
// Queues can be messed up by a maliciouse process (control structures or changing an element after adding)


mod api;
use api::*;
use clap::Parser;




fn a(arg: u32) -> u32 {
    arg + 3
}

fn b(arg: u32) -> u32 {
    arg * 2
}

fn c() -> u32 {
    4
}

fn algo(){
     let first = c();
     let second = b(first);
     let res = a(second);
}



fn wrapper_a(arg_rx: &mut dyn Receiver<u32>, ret_tx: &mut dyn Sender<u32>) {
    let arg = arg_rx.recv().unwrap();
    let res = a(arg);
    ret_tx.send(res).unwrap();
}

fn wrapper_b(arg_rx: &mut dyn Receiver<u32>, ret_tx: &mut dyn Sender<u32>) {
    println!("Waiting for c");
    let arg = arg_rx.recv().unwrap();
    println!("Received from c {:?}", arg);
    let res = b(arg);
    ret_tx.send(res).unwrap();
}


fn wrapper_c(ret_tx: &mut dyn Sender<u32>) {
    let res = c();
    println!("Sending from c: {:?}", res);
    ret_tx.send(res).unwrap();
    println!("Sent from c");
}

fn untyped_wrapper_a(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) {
    let mut arg_rx = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_a(&mut arg_rx, &mut ret_tx)
}


fn untyped_wrapper_b(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) {
    let mut arg_rx = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_b(&mut arg_rx, &mut ret_tx)
}

fn untyped_wrapper_c(tx_fds: &Vec<i32>) {
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_c(&mut ret_tx)
}

pub fn try_dispatch(args: &Args) {
    let command = if let Some(command) = &args.command {
        command
    } else {
        return;
    };

    match command.as_str() {
        "spawn" => {},
        _ =>  return
    }

    if let Some(function) = &args.function {
        match function.as_str() {
            "a" => untyped_wrapper_a(&args.receive_channels, &args.send_channels),
            "b" => untyped_wrapper_b(&args.receive_channels, &args.send_channels),
            "c" => untyped_wrapper_c(&args.send_channels),
            _ => unimplemented!()
        };
    }

    std::process::exit(0);
}



fn main() {
    let args = Args::parse();

    try_dispatch(&args);

    let b_c = PipeChannel::new();
    let a_b = PipeChannel::new();
    let mut main_a = PipeChannel::new();
    
    let mut a = launch_process("a", vec![&a_b], vec![&main_a]).unwrap();
    let mut b = launch_process("b", vec![&b_c], vec![&a_b]).unwrap();
    let mut c = launch_process("c", vec![], vec![&b_c]).unwrap();

    let result : i32 = main_a.recv().unwrap();

    println!("Result is: {}", result);

    a.wait().unwrap();
    b.wait().unwrap();
    c.wait().unwrap();
}

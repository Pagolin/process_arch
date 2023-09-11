// Currently we use pipes, implying memory copies. For less overhead but also less protection we could use 
// shared memory queues loosing isolation but gaining performance.
// Queues can be messed up by a maliciouse process (control structures or changing an element after adding)
mod api;

use api::*;
use std::io::Error as IOError;
use clap::Parser;

mod funs;
use funs::*;


fn input_fun(y:i32) -> i32 {
    let mut s: State = State::new_state(y);
    let stream: Vec<i32> = iter_i32();
    for e in stream {
        let e1: i32 = e;
        s.gs(e1);
    }
    s.gs(5)
}

fn task_0(s_0_1_1_rx: &mut dyn Receiver<State>, b_0_0_tx: &mut dyn Sender<i32>) -> Result<(), IOError>  {
    loop {
        let mut var_0 = s_0_1_1_rx.recv()?;
        println!("Task0 received State");
        let b_0_0 = var_0.gs(5);
        b_0_0_tx.send(b_0_0)?;
        println!("Task0 sent the result");
        ()
      }
    Ok(())
}

fn task_1(ctrl_0_0_tx: &mut dyn Sender<(bool, usize)>, d_1_tx: &mut dyn Sender<i32> ) -> Result<(), IOError> {
    let mut stream_0_0_0 = iter_i32();
      let hasSize =
        {
          let tmp_has_size = stream_0_0_0.iter().size_hint();
          tmp_has_size.1.is_some()
        };
    println!("Task1 about to return Ok");
      Ok(if hasSize {
        let size = stream_0_0_0.len();
        let ctrl = (true, size);
        ctrl_0_0_tx.send(ctrl)?;
        for d in stream_0_0_0 { d_1_tx.send(d)?; println!("Task1 sent {d}"); () }
      } else {
        let mut size = 0;
        for d in stream_0_0_0 {
          d_1_tx.send(d)?;
          //println!("Task1 sent {d}");
          let ctrl = (false, 1);
          ctrl_0_0_tx.send(ctrl)?;
          size = size + 1;
          ()
        };
        let ctrl = (true, 0);
        ctrl_0_0_tx.send(ctrl)?;
        ()
      })
}

// I add I manually here, because it's an env parameter and ignored in Shared Memory code generation
fn task_2(env_i: &mut dyn Receiver<i32>, s_0_0_1_tx: &mut dyn Sender<State>)-> Result<(), IOError> {
    let i = env_i.recv()?;
    let s_0_0_1 = State::new_state(i);
    s_0_0_1_tx.send(s_0_0_1)?;
    println!("Task2 sent state ");
    Ok(())
}

fn task_3(s_0_0_1_rx: &mut dyn Receiver<State>,
          ctrl_0_0_rx: &mut dyn Receiver<(bool, usize)>,
          d_1_rx: &mut dyn Receiver<i32>,
          s_0_1_1_tx: &mut dyn Sender<State>)
-> Result<(), IOError>
{
    loop {
        let mut renew = false;
        println!("Task3 tries to receive state");
        let mut s_0_0_1_0 = s_0_0_1_rx.recv()?;
        println!("Task3 received state  {:?}", s_0_0_1_0);
        while !renew {
          let sig = ctrl_0_0_rx.recv()?;
          let count = sig.1;
          println!("We need data {count} times ");
          for _ in 0 .. count {
            println!("Task3 needs data");
            let var_1 = d_1_rx.recv()?;
            println!("Task3 received data {var_1}");
            s_0_0_1_0.gs(var_1);
            //println!("and did the calculation");
            ()
          };
          println!("Task3 left the for loop");
          let renew_next_time = sig.0;
          renew = renew_next_time;
          ()
        };
        s_0_1_1_tx.send(s_0_0_1_0)?;
        println!("Task3 sent state");
        ()
      }
    Ok(())
}

fn untyped_wrapper_0(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>)-> Result<(), IOError> {
    let mut rx_0  = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut tx_0 = FileSender::from_raw_fd(tx_fds[0]);
    task_0(&mut rx_0, &mut tx_0)
}

fn untyped_wrapper_1(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) -> Result<(), IOError> {
    let mut tx_0  = FileSender::from_raw_fd(tx_fds[0]);
    let mut tx_1 = FileSender::from_raw_fd(tx_fds[1]);
    task_1(&mut tx_0, &mut tx_1)
}

fn untyped_wrapper_2(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) -> Result<(), IOError> {
    let mut rx_0 = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut tx_0  = FileSender::from_raw_fd(tx_fds[0]);
    task_2(&mut rx_0, &mut tx_0)
}

fn untyped_wrapper_3(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>)-> Result<(), IOError> {
    let mut rx_0  = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut rx_1 = FileReceiver::from_raw_fd(rx_fds[1]);
    let mut rx_2 = FileReceiver::from_raw_fd(rx_fds[2]);
    let mut tx_0  = FileSender::from_raw_fd(tx_fds[0]);
    task_3(&mut rx_0, &mut rx_1, &mut rx_2, &mut tx_0)
}



pub fn try_dispatch(args: &Args) {
    let command = if let Some(command) = &args.command {
        command
    } else {
        return;
    };

    match command.as_str() {
        "spawn" => {}
        _ => return
    }
    
    if let Some(function) = &args.function {
        match function.as_str() {
            "task_0" => untyped_wrapper_0(&args.receive_channels, &args.send_channels),
            "task_1" => untyped_wrapper_1(&args.receive_channels, &args.send_channels),
            "task_2" => untyped_wrapper_2(&args.receive_channels, &args.send_channels),
            "task_3" => untyped_wrapper_3(&args.receive_channels, &args.send_channels),
            _ => unimplemented!()
        }.expect("We got an error from one of the nodes");
    }

    std::process::exit(0);
}


fn main() {
    let args = Args::parse();

    try_dispatch(&args);

    let mut env_i = PipeChannel::new();
    let mut b_0_0 = PipeChannel::new();
    let s_0_0_1 = PipeChannel::new();
    let ctrl_0_0 = PipeChannel::new();
    let d_1 = PipeChannel::new();
    let s_0_1_1 = PipeChannel::new();

    // we need to send the arguments of the original algorithm from main
    FileSender::from_raw_fd(env_i.tx_fd()).send(42).expect("Sending algo argument failed");
    println!("About to launch");
    let mut task_0 = launch_process("task_0", vec![&s_0_1_1], vec![&b_0_0]).unwrap();
    let mut task_1 = launch_process("task_1", vec![], vec![&ctrl_0_0, &d_1]).unwrap();
    let mut task_2 = launch_process("task_2", vec![&env_i], vec![&s_0_0_1]).unwrap();
    let mut task_3 = launch_process("task_3", vec![&s_0_0_1, &ctrl_0_0, &d_1], vec![&s_0_1_1]).unwrap();
    println!("Launched");
    let result: i32 = b_0_0.recv().unwrap();
    println!("Result is: {}", result);

    s_0_0_1.close();
    s_0_1_1.close();
    ctrl_0_0.close();
    task_0.wait().unwrap();
    task_1.wait().unwrap();
    task_2.wait().unwrap();
    task_3.wait().unwrap();

}

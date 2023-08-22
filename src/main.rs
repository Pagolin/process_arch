use std::process::{Command, Stdio};
use std::{env, process, io::Write};
use ipc_channel::ipc::{channel};
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};


fn spawn<Ti, To>(function: &str, sender:IpcSender<To>, receiver: IpcReceiver<Ti>) -> process::Child {
    let current_dir = env::current_exe().unwrap();
    // This assumes the executables of the nodes are in the same crate as the main process
    // So we get the directory of the main process,
    // split the name of the main executable and append the name of the current function
    let function_crate_path = current_dir
        .parent()
        .unwrap()
        .join(function);
    let mut child = Command::new(function_crate_path)
        //.arg("frobnicate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute server process");
    let flat_sender = bincode::serialize(&sender).unwrap();
    let mut child_stdin = child.stdin.take().unwrap();
    //child_stdin.write_all(&flat_sender).unwrap();
    child.stdin.insert(child_stdin);
    child
}

// This seems to be a simpler way to spawn rust code https://github.com/servo/ipc-channel/blob/fdeb9231b193b75b2daec173c6e24c2ad799927c/src/test.rs#L118
// Used for example like this : https://github.com/servo/ipc-channel/blob/fdeb9231b193b75b2daec173c6e24c2ad799927c/src/platform/test.rs#L684
fn main(){
    let (tx_result, rx_result) = channel::<String>().unwrap();
    let (tx_algo1, rx_algo1) = channel::<String>().unwrap();
    tx_result.send("Frobnicate".to_string());
    let mut child = spawn("algo1", tx_result, rx_algo1);
    let output = child.wait().expect("failed to wait on child");
    println!("output = {:?}", output);
    let result = rx_result.recv().unwrap();
    println!("sending worked because I got = {:?}", result);
}
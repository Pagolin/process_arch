use std::{env, process, thread, time};
use std::process::{Command, Stdio};
use ipc_channel::platform::{OsIpcOneShotServer, OsIpcSender};


fn main() {
    let mut args: Vec<String> = vec![];
    let ten_millis = time::Duration::from_millis(100);
    thread::sleep(ten_millis);
    while args.len() < 2 {
        args = env::args().collect();
        println!("{:?}", args);
        thread::sleep(ten_millis);
    }
    let input = &args[1];
    println!("Got {} from other process", input);
}


fn command_wrapper(test_name: &str, server_args: &[(&str, &str)]) -> process::Child {
    Command::new(env::current_exe().unwrap())
        .arg(test_name)
        .args(server_args.iter()
                         .map(|&(ref name, ref val)| format!("channel_name-{}:{}", name, val)))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to execute server process")
}
pub fn get_channel_name_arg(which: &str) -> Option<String> {
    for arg in env::args() {
        let arg_str = &*format!("channel_name-{}:", which);
        if arg.starts_with(arg_str) {
            return Some(arg[arg_str.len()..].to_owned());
        }
    }
    None
}

#[test]
fn spawning_function(){
    let data: &[u8] = b"1234567";

    let channel_name = get_channel_name_arg("server");
    if let Some(channel_name) = channel_name {
        let tx = OsIpcSender::connect(channel_name).unwrap();
        tx.send(data, vec![], vec![]).unwrap();

        unsafe { libc::exit(0); }
    }

    let (server, name) = OsIpcOneShotServer::new().unwrap();
    let mut child_pid = command_wrapper("cross_process_spawn", &[("server", &*name)]);

    let (_, received_data, received_channels, received_shared_memory_regions) =
        server.accept().unwrap();
    child_pid.wait().expect("failed to wait on child");
    assert_eq!((&received_data[..], received_channels, received_shared_memory_regions),
               (data, vec![], vec![]));
}


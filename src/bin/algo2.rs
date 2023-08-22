use std::{env, process, thread, time};
use std::process::{Command, Stdio};
use os_pipe::{PipeReader, PipeWriter};
use std::io::Error as IoError;
use std::error::Error;
use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::FromRawFd;
use std::marker::PhantomData;
use command_fds::{CommandFdExt, FdMapping};
use std::os::fd::AsRawFd;

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

trait Sender<T: Sized> {
    fn send(&mut self, msg: T) -> Result<(), IoError>;
}

struct FileSender {
    file: File,
}

impl FileSender {
    fn from_raw_fd(raw_fd: i32) -> Self {
        Self {
            file: unsafe {File::from_raw_fd(raw_fd)}
        }
    }
}

impl<T: Copy> Sender<T> for FileSender {
    fn send(&mut self, msg: T) -> Result<(), IoError>  {
        self.file.write(unsafe { any_as_u8_slice(&msg)})?;
        Ok(())
    }
}


trait Receiver<T: Sized> {
    fn recv(&mut self) -> Result<T, IoError>;
}

struct FileReceiver<T> {
    file: File,
    phantom: PhantomData<T>,
}

impl<T> FileReceiver<T> {
    fn from_raw_fd(raw_fd: i32) -> Self {
        Self {
            file: unsafe {File::from_raw_fd(raw_fd)},
            phantom: PhantomData,
        }
    }
}

impl<T: Copy> Receiver<T> for FileReceiver<T> {
    fn recv(&mut self) -> Result<T, IoError> {
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;

        let ret = unsafe {
            *(buffer.as_ptr() as *const T)
        };

        Ok(ret)
    }
}


fn c() -> u32 {
    3
}

fn wrapper_c(ret_tx: &mut dyn Sender<u32>) {
    let res = c();
    ret_tx.send(res).unwrap();
}

fn untyped_wrapper_c(tx_fds: &Vec<i32>) {
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_c(&mut ret_tx)
}

fn b(arg: u32) -> u32 {
    arg * 2
}

fn wrapper_b(arg_rx: &mut dyn Receiver<u32>, ret_tx: &mut dyn Sender<u32>) {
    let arg = arg_rx.recv().unwrap();
    let res = b(arg);
    ret_tx.send(res).unwrap();
}

fn untyped_wrapper_b(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) {
    let mut arg_rx = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_b(&mut arg_rx, &mut ret_tx)
}

fn a(arg: u32) -> u32 {
    arg + 3
}

fn wrapper_a(arg_rx: &mut dyn Receiver<u32>, ret_tx: &mut dyn Sender<u32>) {
    let arg = arg_rx.recv().unwrap();
    let res = a(arg);
    ret_tx.send(res).unwrap();
}

fn untyped_wrapper_a(rx_fds: &Vec<i32>, tx_fds: &Vec<i32>) {
    let mut arg_rx = FileReceiver::from_raw_fd(rx_fds[0]);
    let mut ret_tx = FileSender::from_raw_fd(tx_fds[0]);
    wrapper_a(&mut arg_rx, &mut ret_tx)
}

// fn algo(){
//     let first = c();
//     let second = b(first);
//     let final = a(second());
// }

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    command: String,

    /// Number of times to greet
    #[arg(short, long)]
    function: Option<String>,

    /// Map of channels with file descriptor names
    #[arg(short = 'R',  number_of_values = 1)]
    receive_channels: Vec<i32>,

    #[arg(short = 'S',  number_of_values = 1)]
    send_channels: Vec<i32>
}

fn try_dispatch(args: &Args) {
    if args.command != "spawn" {
        return
    }

    // let receivers : Vec<Sender> = args.receive_channels
    //         .iter()
    //         .map(|raw_fd| {
    //             FileSender::from_raw_fd(raw_fd)
    //         })
    //         .collect();


    // let senders : Vec<File> = args.send_channels
    //         .iter()
    //         .map(|chan_fd| {
    //             unsafe {File::from_raw_fd(*chan_fd)}
    //         })
    //         .collect();

    if let Some(function) = &args.function {
        match function.as_str() {
            "a" => untyped_wrapper_a(&args.receive_channels, &args.send_channels),
            "b" => untyped_wrapper_b(&args.receive_channels, &args.send_channels),
            "c" => untyped_wrapper_c(&args.send_channels),
            _ => unimplemented!()
        };
    }
}

struct PipeChannel {
    rx: PipeReader,
    tx: PipeWriter
}

impl PipeChannel {
    fn new() -> Self {
        let (mut reader, mut writer) = os_pipe::pipe().unwrap();
        Self { rx: reader, tx: writer }
    }

    fn rx_fd(&self) -> i32 {
        self.rx.as_raw_fd()
    }

    fn tx_fd(&self) -> i32 {
        self.tx.as_raw_fd()
    }

    fn recv<T: Copy>(&mut self) -> Result<T, IoError> {
        let mut buffer = Vec::new();
        self.rx.read_to_end(&mut buffer)?;

        let ret = unsafe {
            *(buffer.as_ptr() as *const T)
        };

        Ok(ret)
    }
}

// Pipe first -> (rx, tx)
// rx -> a
// tx -> b
// b -> a

fn launch_process(name: &str, rx_channels: Vec<&PipeChannel>, tx_channels: Vec<&PipeChannel>)
     -> Result<process::Child, IoError>
{
    let mut command = Command::new("/proc/self/exe");

    let mut fd_mappings = vec![];

    for rx_channel in rx_channels.iter() {
        command.arg("-R").arg(format!("{}", rx_channel.rx_fd()));

        fd_mappings.push(
            FdMapping {
                parent_fd: rx_channel.tx_fd(),
                child_fd: rx_channel.rx_fd()
            }            
        );
    }

    for tx_channel in tx_channels.iter() {
        command.arg("-S").arg(format!("{}", tx_channel.tx_fd()));

        fd_mappings.push(
            FdMapping {
                parent_fd: tx_channel.rx_fd(),
                child_fd: tx_channel.tx_fd()
            }            
        );
    }

    command.fd_mappings(fd_mappings);

    command.spawn()
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

fn old_main() {
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

// #[test]
// fn spawning_function(){
//     let data: &[u8] = b"1234567";

//     let channel_name = get_channel_name_arg("server");
//     if let Some(channel_name) = channel_name {
//         let tx = OsIpcSender::connect(channel_name).unwrap();
//         tx.send(data, vec![], vec![]).unwrap();

//         unsafe { libc::exit(0); }
//     }

//     let (server, name) = OsIpcOneShotServer::new().unwrap();
//     let mut child_pid = command_wrapper("cross_process_spawn", &[("server", &*name)]);

//     let (_, received_data, received_channels, received_shared_memory_regions) =
//         server.accept().unwrap();
//     child_pid.wait().expect("failed to wait on child");
//     assert_eq!((&received_data[..], received_channels, received_shared_memory_regions),
//                (data, vec![], vec![]));
// }

/*

1. Generated binary
  
2. Process clone



Ohua a(b(c())) -> 
  1. Create a
  2. Create b
  3. Create
  4. Connect main -> a
  5. Connect a -> b
  6. Connect b -> c



1. Generated binary approach

fn __main() {
    let chan_c = spawn(c, None);
    let chan_b = spawn(b, chan_c);
    let chan_a = spawn(a, chan_b);
    let chan_main = chan_a.recv();
    
    let res_c = channel_c.unwrap().recv();
    channel_b.unwrap().send(res_c);
    let res_b = channel_b.unwrap().recv();
    return a()
}

fn main() {
    match cmd.function {
        "main" => __main(),
        "a" => a(),
        "b" => b(),
        "c" => c()
    }
}



fn main() {
    let res_c = c()
    let res_b = b(res_c);
    let res_a = a(res_b);
    return res_a;
}

fn main(){
    let procs = [(a, a_tx, b_rx), (b, c_rx, b_tx), (c, c_tx)];
    let c_tx, c_rx = channel();
    let b_tx, b_rx = channel();
    let a_tx, a_rx = channel();
    handles = [];
    
    for proc in procs{
        handles.add(process!(proc);)
    }

    final = a_rx.recv();

    for handle in handles {
        handle.join();
        handle.terminate();
    }
    
}

Ohua --(Trait API)--> generate the main function
     --(start) --> compile


2. Process clone

Box<&dyn Sender>

trait Sender<T> {
    fn send(&self, i:T)
}

*/



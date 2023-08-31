use std::os::unix::io::FromRawFd;
use std::marker::PhantomData;
use std::fs::File;
use std::io::Error as IoError;
use std::{process};
use std::process::{Command};
use os_pipe::{PipeReader, PipeWriter};
use clap::Parser;

use std::io::{Read, Write};

use command_fds::{CommandFdExt, FdMapping};
use std::os::fd::AsRawFd;


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

pub trait Sender<T: Sized> {
    fn send(&mut self, msg: T) -> Result<(), IoError>;
}


pub trait Receiver<T: Sized> {
    fn recv(&mut self) -> Result<T, IoError>;
}


pub struct FileSender {
    file: File,
}

impl FileSender {
    pub fn from_raw_fd(raw_fd: i32) -> Self {
        // println!("{}", raw_fd);
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



pub struct FileReceiver<T> {
    file: File,
    phantom: PhantomData<T>,
}

impl<T> FileReceiver<T> {
    pub fn from_raw_fd(raw_fd: i32) -> Self {
        // println!("{}", raw_fd);
        Self {
            file: unsafe {File::from_raw_fd(raw_fd)},
            phantom: PhantomData,
        }
    }
}

impl<T: Copy> Receiver<T> for FileReceiver<T> {
    fn recv(&mut self) -> Result<T, IoError> {
        //FIXME: We need a read of appropriate length i.e. size of T
        let mut buffer = [0; 1024];
        self.file.read(&mut buffer)?;

        let ret = unsafe {
            *(buffer.as_ptr() as *const T)
        };

        Ok(ret)
    }
}




pub struct PipeChannel<T: Copy + Sized> {
    rx: PipeReader,
    tx: PipeWriter,
    _phantom: PhantomData<T>
}

impl<T: Copy + Sized> PipeChannel<T> {

    pub fn new() -> Self {
        let (reader, writer) = os_pipe::pipe().unwrap();
        Self { rx: reader, tx: writer , _phantom:PhantomData}
    }

    fn rx_fd(&self) -> i32 {
        self.rx.as_raw_fd()
    }

    fn tx_fd(&self) -> i32 {
        self.tx.as_raw_fd()
    }

    pub fn recv(&mut self) -> Result<T, IoError> {
        let mut buffer = [0; 1024];
        self.rx.read(&mut buffer)?;

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

pub fn launch_process(name: &str, rx_channels: Vec<&PipeChannel>, tx_channels: Vec<&PipeChannel>)
     -> Result<process::Child, IoError>
{
    let mut command = Command::new("/proc/self/exe");

    let mut fd_mappings = vec![];

    command.arg("--command").arg("spawn").arg("--function").arg(name);

    for rx_channel in rx_channels.iter() {
        command.arg("-R").arg(format!("{}", rx_channel.tx_fd()));

        fd_mappings.push(
            FdMapping {
                parent_fd: rx_channel.rx_fd(),
                child_fd: rx_channel.tx_fd()
            }            
        );
    }

    for tx_channel in tx_channels.iter() {
        command.arg("-S").arg(format!("{}", tx_channel.rx_fd()));

        fd_mappings.push(
            FdMapping {
                parent_fd: tx_channel.tx_fd(),
                child_fd: tx_channel.rx_fd()
            }            
        );
    }

    command.fd_mappings(fd_mappings).unwrap();

    command.spawn()
}


/// Helper to parse the command line arguments we get when spanwing a process
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// In our case the command should be spawn if where in the main process and kick of the nodes 
    #[arg(short, long)]
    pub command: Option<String>,

    /// The name of the function to execute in the new process
    #[arg(short, long)]
    pub function: Option<String>,

    /// Map of receiving channels with file descriptor names
    #[arg(short = 'R',  number_of_values = 1)]
    pub receive_channels: Vec<i32>,

    /// Map of sendinging channels with file descriptor names
    #[arg(short = 'S',  number_of_values = 1)]
    pub send_channels: Vec<i32>
}

use std::env;
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use std::io::{Read};

fn prompt_ctn<R>(question: String, ctn: F) -> R
where F: Fn(T) -> R
{
    let val = prompt_for_input(question);
    ctn(val)
}

fn prompt_for_input(question:String) -> T { /*get user input and convert*/}

fn server_function(first: T, second: T) -> TR {/*do server stuff*/}

fn main() {
    let () = prompt_ctn("First?".into(),
     |fst| prompt_ctn("Second?".into(),
                |snd| {
                    server_function(fst, snd);
                    println!("Done with {:?}", result);
                }
     ));
}

/*
fn main() {
    // Read the serialized sender from stdin
    let mut sender_bytes = Vec::new();
    std::io::stdin().read_to_end(&mut sender_bytes).expect("Failed to read sender from stdin");
    println!("Algo 1 read {:?} bytes from stdin", sender_bytes.len());
    // Deserialize the sender
    let sender: IpcSender<String> = bincode::deserialize(&sender_bytes).expect("Failed to deserialize sender");

    // Send a message back to the parent process
    sender.send("Message from Algo1".to_string()).expect("Failed to send message to parent");

    //let args: Vec<String> = env::args().collect();
    //let query = &args[1];
    println!("Algo 1 worked so far");
}

 */
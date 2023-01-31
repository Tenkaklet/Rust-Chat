use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpStream, SocketAddr, IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 32;
fn main() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000);
    let mut client = TcpStream::connect(socket).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");
    println!("the address of client is {:?}", socket);

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();

                let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                let test = TcpStream::peer_addr(&client).unwrap();
                println!("message recv {msg} from {test}");
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
                println!("message sent well");
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message to the server:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":exit" || tx.send(msg).is_err() {break} // ":quit" is to exit the program
    }
    println!("bye bye!");

}

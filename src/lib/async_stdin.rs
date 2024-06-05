use std::io::{BufRead, BufReader, Stdin};
use tokio::sync::mpsc;

pub fn stdin_channel(stdin: Stdin) -> mpsc::Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>(1);
    let buff = BufReader::new(stdin);
    std::thread::spawn(move || read_loop(buff, tx));
    rx
}

fn read_loop<R>(reader: R, tx: mpsc::Sender<u8>)
    where
        R: BufRead,
{
    let mut bytes = reader.bytes();
    loop {
        if let Some(Ok(byte)) = bytes.next() {
            let _ = tx.blocking_send(byte);
        }
    }
}
use std::{
    io::{Read, Write},
    net::TcpListener,
};

const BINARY: &[u8] = include_bytes!("../firmware.bin");
fn main() {
    println!("BINARY_SIZE: {}", BINARY.len());

    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    for stream in listener.incoming() {
        println!("Connection");
        let mut stream = stream.unwrap();

        let buf = (BINARY.len() as u32).to_le_bytes();
        _ = stream.write_all(&buf);

        let chunks = BINARY.chunks(4096 * 2);
        let mut buf = [0; 1];
        for chunk in chunks {
            println!("Writing: {}", chunk.len());

            _ = stream.write_all(chunk);
            _ = stream.read_exact(&mut buf);

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

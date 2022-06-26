use std::{
    io,
    net::{TcpListener, TcpStream},
    thread,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9090")?;
    let client = TcpStream::connect("103.86.44.158:80")?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut stream_clone = stream.try_clone()?;
        let mut client = client.try_clone()?;
        let mut client_clone = client.try_clone()?;
        thread::spawn(move || io::copy(&mut stream, &mut client).unwrap());
        io::copy(&mut client_clone, &mut stream_clone).unwrap();
    }
    Ok(())
}

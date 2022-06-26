#![deny(warnings)]

use std::{
    error::Error,
    io::{self, Read, Write},
    net::{Ipv4Addr, Ipv6Addr, TcpListener, TcpStream},
    thread,
};

// curl --socks5-hostname 127.0.0.1:1080 baidu.com
fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = "127.0.0.1:7080";
    let listener = TcpListener::bind(listen_addr)?;
    for strem in listener.incoming() {
        match strem {
            Ok(stream_data) => {
                println!("New connection");
                thread::spawn(move || {
                    if let Err(err) = handle_connection(&stream_data) {
                        println!("error: {:?}", err)
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(stream_data: &TcpStream) -> Result<(), Box<dyn Error>> {
    println!("src: {}", stream_data.peer_addr()?);
    let mut src_reader = stream_data.try_clone()?;
    let mut src_writer = stream_data.try_clone()?;
    do_greeting(&mut src_reader, &mut src_writer)?;
    let dst = parse_dst(&mut src_reader)?;
    println!("dst: {dst}");
    let dst_stream = TcpStream::connect(dst)?;
    let mut dst_reader = dst_stream.try_clone()?;
    let mut dst_writer = dst_stream.try_clone()?;

    src_writer.write(&[0x05])?;

    src_writer.write(&[0x00])?;
    src_writer.write(&[0x00])?;

    src_writer.write(&[0x01])?;
    src_writer.write(&[0x00])?;
    src_writer.write(&[0x00])?;
    src_writer.write(&[0x00])?;
    src_writer.write(&[0x00])?;

    src_writer.write(&[0x00])?;
    src_writer.write(&[0x00])?;
    thread::spawn(move || {
        io::copy(&mut src_reader, &mut dst_writer).ok();
    });

    io::copy(&mut dst_reader, &mut src_writer).ok();
    Ok(())
}

fn parse_dst(src_reader: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    let mut buf = [0u8; 256];
    src_reader.read_exact(&mut buf[..1])?;
    if buf[0] != 0x05 {
        return Err("Invalid version".into());
    };

    src_reader.read_exact(&mut buf[..1])?;
    if buf[0] != 0x01 {
        return Err("Invalid cmd".into());
    };
    src_reader.read_exact(&mut buf[..1])?;
    let host = match buf[0] {
        0x01 => {
            src_reader.read_exact(&mut buf[..4])?;
            Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]).to_string()
        }
        0x03 => {
            src_reader.read_exact(&mut buf[..1])?;
            let len = buf[0] as usize;
            src_reader.read_exact(&mut buf[..len])?;
            String::from_utf8_lossy(&buf[..len]).to_string()
        }
        0x04 => {
            src_reader.read_exact(&mut buf[..16])?;
            Ipv6Addr::new(
                ((buf[0x00] as u16) << 8) | (buf[0x01] as u16),
                ((buf[0x02] as u16) << 8) | (buf[0x03] as u16),
                ((buf[0x04] as u16) << 8) | (buf[0x05] as u16),
                ((buf[0x06] as u16) << 8) | (buf[0x07] as u16),
                ((buf[0x08] as u16) << 8) | (buf[0x09] as u16),
                ((buf[0x0a] as u16) << 8) | (buf[0x0b] as u16),
                ((buf[0x0c] as u16) << 8) | (buf[0x0d] as u16),
                ((buf[0x0e] as u16) << 8) | (buf[0x0f] as u16),
            )
            .to_string()
        }
        _ => {
            return Err("Invalid address type".into());
        }
    };

    src_reader.read_exact(&mut buf[..2])?;
    let port = ((buf[0] as u16) << 8) | (buf[1] as u16);
    let dst = format!("{}:{}", host, port);
    Ok(dst)
}

fn do_greeting(
    src_reader: &mut TcpStream,
    src_writer: &mut TcpStream,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 256];
    src_reader.read_exact(&mut buf[..1])?;
    if buf[0] != 0x05 {
        return Err("Invalid version".into());
    };
    src_writer.write(&[0x05])?;
    src_reader.read_exact(&mut buf[..1])?;
    let nauth = buf[0] as usize;
    src_reader.read_exact(&mut buf[..nauth])?;
    src_writer.write(&[0x00])?;
    println!("greeting done!");
    Ok(())
}

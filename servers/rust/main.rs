#![feature(libc)]
#![feature(tcp)]

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io;
use std::env;
use std::mem;
use std::str::FromStr;
use std::io::Read;
use std::io::Write;

extern crate libc;

fn main() {
    let port_ : Option<String> = env::args().skip(2).next();
    if let None = port_ {
        println!("usage: {} -p port", env::args().next().unwrap());
        return;
    }

    let port = u16::from_str(&port_.unwrap());
    if let Err(ref e) = port {
        println!("invalid port number given: {}", e);
        return;
    }

    let listener = TcpListener::bind(("127.0.0.1", port.unwrap()));
    if let Err(ref e) = listener {
        println!("failed to listen on port: {}", e);
        return;
    }

    for stream in listener.unwrap().incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut challenge : u32 = 0;
    let buf : &mut [u8; 4] = unsafe{ mem::transmute(&mut challenge) };
    let mut nread;

    let _ = stream.set_nodelay(true);

    loop {
        nread = 0;
        while nread < buf.len() {
            match stream.read(&mut buf[nread..]) {
                Ok(n) if n == 0 => return,
                Ok(n) => nread += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }

        challenge = u32::from_be(challenge);
        if challenge == 0 {
            unsafe { libc::exit(0 as libc::c_int); }
        }
        challenge = u32::to_be(challenge + 1);

        let mut nwritten = 0;
        while nwritten < buf.len() {
            match stream.write(&buf[nwritten..]) {
                Ok(n) => nwritten += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }

        let _ = stream.flush();
    }
}

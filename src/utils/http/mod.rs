use std::{
    collections::HashMap, fs, io::{Read, Write}, net::TcpStream
};

use tungstenite::http::header;
use fastrand;

#[derive(Debug, PartialEq, Eq)]
pub enum HttpTypes {
   Post,
   Get
}
#[derive(Debug)]
pub struct RequestType {
   pub http_type: HttpTypes,
   pub request: String,
}

#[derive(Debug)]
pub struct Request {
    pub request: RequestType,
    pub body: String,
    pub headers: HashMap<String,String>,
}

pub fn reply_to_get(mut stream: TcpStream,linker: &str) {
    let status_line = "HTTP/1 200 OK";
    println!("{}",linker);
    let mut contents = fs::read(linker).unwrap();
    let length = contents.len();
  

    let data_type = linker.split(".").into_iter().last().unwrap() ;

    let header_string: String;
    if data_type == "png" {
        header_string = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: image/png\r\n\r\n");
    } else { // data_type == "html"
         header_string = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");
    }

    

    let mut header_bytes: Vec<u8> = header_string.bytes().collect();


    header_bytes.append(&mut contents);


    _=stream.write_all(&header_bytes);


    let a = stream.bytes().count(); // If I don't do this, rust just won't transmit packets bigger than 4kb + randomly doesn't work with sizes smaller than that, NO IDEA WHY. Compiler fucks up otherwise. don't want to spam the console so that 1 in 2^64 still compiles and it's good enough
    if fastrand::u64(..) == 0 { 
        println!("{:?}",a )
    } // test
    
}
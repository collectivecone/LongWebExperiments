#![allow(dead_code)]
#![allow(unused_braces)]
#![allow(unused_variables)]

use std::net::TcpStream;
    

use crate::utils::{
    http::{
        reply_to_get, HttpTypes, Request
    }
};


pub fn main() {
   
}

pub fn http_request(stream: TcpStream, request: Request) {
    match request.request.http_type {
        HttpTypes::Get => reply_to_get(stream,"src/experiments/base/website.html"),
        _ => {}
    }
}

pub fn websocket_request(stream: TcpStream, request: Request) {
  
}
use core::net;
use std::{
    net::{TcpListener, TcpStream,SocketAddr},
    collections::HashMap,
    thread::spawn,
};


mod utils;
mod experiments;

use utils::http::{
    Request,
    RequestType,
    HttpTypes,
    reply_to_get,
};

pub mod settings{
    use std::sync::RwLock;

    pub static GLOBAL_SETTINGS: RwLock<SettingsStruct> = RwLock::new(SettingsStruct{
        ignore_multiple_connections_per_ip: true,
    } );
    pub struct SettingsStruct {
        pub ignore_multiple_connections_per_ip: bool,
    }
}

fn main() {
    startup_experiments();
    perm_http_receiver();
}

fn startup_experiments() {
    spawn(|| crate::experiments::restrictions::main());
}

fn perm_http_receiver() {
    let addrs = [
        SocketAddr::from(([0, 0, 0, 0], 80)),
        SocketAddr::from(([0, 0, 0, 0], 8000)),
        SocketAddr::from(([0, 0, 0, 0], 8080)),
        SocketAddr::from(([127, 0, 0, 1], 8000)),
        SocketAddr::from(([127, 0, 0, 1], 8080)),
    ];
    let listener = TcpListener::bind(&addrs[..]).unwrap();
    println!("Running server at {}", listener.local_addr().unwrap());
    println!("Connecting through ip rather than cloudflare/an url is buggy and requires refreshing on chrome for whatever reason but does function (if anyone figures that out, do a pull request on the git)");
    println!("Your firewall might also do that ");
    if let net::IpAddr::V4(a) =  listener.local_addr().unwrap().ip() {
        if listener.local_addr().unwrap().port() != 80  {
           
            if !a.is_unspecified() {
                println!("Server is only running locally");
                println!("Try running server with admin permissions (or any permissions you can give) attempt connect to internet");
                println!("Typing {} into the browser will allow you to see what the website would look like but nobody else is able to connect", listener.local_addr().unwrap());
                println!("Perhaps try using sudo if your in linux or running as administrator in windows");
            } else {
                println!("The program has privillages to connect to the internet but is unable to connect to port 80 which is the standard for http and websites in general");
                println!("Perhaps try using sudo if your in linux or running as administrator in windows");
                println!("Anyone is the internet is able to connect but they must put :{} after the url or ip address in order to connect", listener.local_addr().unwrap().port());
                println!("There are ways in either windows or linux to map port 80 to ones the program can access");
                println!("Perhaps change your computer's settings so non privillaged users can access ports before 1024");
                println!("linux:https://superuser.com/questions/710253/allow-non-root-process-to-bind-to-port-80-and-443/892391#892391")
            }
        }
    };

    for stream in listener.incoming() { 

        let mut stream = stream.unwrap();
        if let Some(request) = get_body_and_headers(&mut stream) {
            if request.headers.get("Upgrade").unwrap_or(&String::new()) == "websocket" {
                websocket_handling(stream,request);
            } else {
                website_handling(stream,request);
            }
        };
    }
}


fn websocket_handling(stream: TcpStream, request: Request ) {
    let link = request.request.request.clone();

    if link == "/restrictions" {
        crate::experiments::restrictions::websocket_request(stream,request);
    }
}


fn website_handling(stream: TcpStream, request: Request) {

    let link = request.request.request.clone();
    
    if link.as_str() == "/style.css" {
        reply_to_get(stream, "style.css");
    } else if link.as_str().trim() == "/favicon.png".trim() {
        reply_to_get(stream, "favicon.png");
    } else if link == "/restrictions" {
        crate::experiments::restrictions::http_request(stream,request);
    } else if link == "/" || link ==  "/base" {
        crate::experiments::base::http_request(stream,request);
    };
   // if link == "/" {
       
 //   }

    

}



pub fn get_body_and_headers(stream: &mut TcpStream) -> Option<Request> { 
    let mut buf = [0; 10000];
    if let Ok(len) = stream.peek(&mut buf) {
      let mut buf = vec![0;len];
      let _ = stream.peek(&mut buf).unwrap();
   
      if let Ok(whole_request) = String::from_utf8(buf.to_vec()) {
        let mut header_str: String = String::new();
        let mut header = HashMap::new();
        let mut request: RequestType = RequestType{http_type : HttpTypes::Get, request: String::new()} ;
    
        let mut lines = whole_request.lines();
     
        loop {
           let line = lines.next();
           match line {
              Some(line) => {
              if line.len() < 3 {
                 break;
              }
           
              header_str.push_str( line);
              header_str.push('\n');
           },
              None => return None,
           }
        }
     
     
        for (i, line) in header_str.lines().into_iter() .enumerate() {
           let thing: Vec<&str> = line.split(" ").collect();
           if i == 0 {
              match thing.get(0) {
                 Some(t) =>  {
                       match t {
                          &"GET" => {request.http_type = HttpTypes::Get}
                          &"POST" => {request.http_type = HttpTypes::Post}
     
     
                          _ => {return None},
                       }
                 },
                 None => {return None}
              }
     
              match thing.get(1) {
                 Some(t) => {request.request = t.to_string() },
                 None => {return None},
              }
           }
           else {
              let mut x = thing.get(0).unwrap_or(&"").to_string();
              let y = thing.get(1).unwrap_or(&"").to_string();
              if !x.is_empty() && !y.is_empty() {
                 x.pop();
     
                 header.insert(x,y);
              }
           }
        }
     
        let mut body = String::new();
        for line in lines {
            body.push_str(line);
        }
        
        return Some(Request{request: request, body: body, headers: header})
      };
    } 
    return None
}


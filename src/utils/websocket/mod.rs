use std::{
    collections::HashMap, net::TcpStream, ops::DerefMut, sync::{
        mpsc::Sender, Mutex, MutexGuard
    }, thread::{sleep,spawn}, time::Duration
};
use tungstenite::{
    WebSocket,
    accept_with_config,
    protocol::WebSocketConfig
};
use fastrand;
use crate::settings::GLOBAL_SETTINGS;

#[derive(Debug)]
pub struct WebsocketData {
    pub msg: tungstenite::Message,
    pub user_id: u64,
}
pub struct NetworkUser {
    pub websocket: WebSocket<TcpStream>,
    pub true_ip: String,
    pub id: u64,
}


pub trait NetworkBased {
    fn network_mut(&mut self) -> &mut NetworkUser;
    fn network(&self) -> &NetworkUser;
}

 
fn get_ip(headers: &HashMap<String,String>, stream : &TcpStream) -> Option<String> {
    let ip = headers.get("CF-Connecting-IP");
    if let Some(ip) = ip {
        return Some(ip.clone());
    } else {
        match  stream.local_addr() {
            Ok(ip) =>   return  Some(ip.ip().to_string()),
            Err(_) => return None
        }
    }
}

fn is_multi_connecting(user_vec: &Vec<impl NetworkBased>, ip_string: &String) -> bool {
    if GLOBAL_SETTINGS.read().unwrap().ignore_multiple_connections_per_ip {return false}

    for user in user_vec {
        if user.network().true_ip == *ip_string {
            return true
        }
    }
    return false
}

pub fn add_new_user<'a>(stream: TcpStream,headers: HashMap<String,String>,guard: &'a mut MutexGuard<Vec<impl NetworkBased>>) -> Option<NetworkUser> {

    if let Some(ip_string) = get_ip(&headers,&stream) {
        let user_vec = (*guard).deref_mut() ;
        if !is_multi_connecting(&user_vec,&ip_string) {
            _ = stream.set_nonblocking(true);
            let websocket = accept_with_config(stream,Some(WebSocketConfig{
                write_buffer_size: 0,
                max_write_buffer_size: usize::MAX,
                max_message_size: None,
                max_frame_size: None,
                accept_unmasked_frames: false,
                 ..Default::default()
            })).unwrap();

            let user = NetworkUser{
                websocket: websocket,
                true_ip: ip_string,
                id: fastrand::u64(..),
            };
            
            return Some(user);
        }
    };
    None
}

pub fn get_user_by_id<U> (users: &mut Vec<U>, id: u64 ) -> Option<&mut U>
where U: NetworkBased
{
    for user in users {
        if user.network().id == id {
            return Some(user)
        };
    };
    None
}

pub fn send_to_user(user: &mut impl NetworkBased, msg: tungstenite::Message) {
    _ = user.network_mut().websocket.send(msg);
} 

pub fn send_to_all_users(user_vec: &mut Vec<impl NetworkBased>, msg: tungstenite::Message) {
    for user in user_vec.iter_mut() {
        _ = user.network_mut().websocket.send(msg.clone());
    }
}



pub fn read_all_inputs(global_users : &'static Mutex<Vec<impl NetworkBased + Sync + Send>>, websocket_sender: Sender<WebsocketData>  ) {
    spawn(move || {
        loop {
            sleep(Duration::from_secs_f32(0.004));
            let mut guard= global_users.lock().unwrap();
            let  users = guard.deref_mut();
            let mut to_delete: Vec<usize> = vec!();
            for (i,user) in users.iter_mut().enumerate() {
                let network_user = user.network_mut();
                loop {
                    match network_user.websocket.read() {
                        Ok(msg) => {
                            match msg {
                                tungstenite::Message::Text(_) => {
                                    _ = websocket_sender.send(WebsocketData{msg: msg,user_id: network_user.id});
                                },
                                _ => {}
                            }
                        }
                        Err(e) => {
                        if e.to_string() == "Trying to work with closed connection" {
                            to_delete.push(i);
                        } 
                        break;
                        }
                    }
                }
            }
            for (i,delete_index) in to_delete.iter().enumerate() {
                users.swap_remove(delete_index - i);
            }
        }
    });
}
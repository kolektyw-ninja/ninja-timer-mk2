
use std::thread::{spawn, JoinHandle};
use crate::state::InputEvent;
use std::sync::mpsc::Sender;

use std::os::unix::net::UnixStream;

use std::io::{BufReader, BufRead, ErrorKind};


const SOCKET_PATH: &'static str = "/tmp/cringed/events.sock";

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) enum EvtType {
    ButtonPress,
    ButtonRelease,
    Overcurrent,
    CriticalError,
    TransportError
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct CringeEvt {
    pub(crate) io_bank_num: u8,
    pub(crate) event_type: EvtType,
    pub(crate) timestamp_ms: u32
}

fn parse_event(event: &str) -> CringeEvt{
    //parse str for event
    let cevt: CringeEvt = serde_json::from_str(event).unwrap_or(CringeEvt {
        io_bank_num: 0,
        event_type: EvtType::TransportError,
        timestamp_ms: 0
    });
    println!("{} {:?} {}", cevt.io_bank_num, cevt.event_type, cevt.timestamp_ms);
    return cevt;
}

pub fn spawn_gpio(sender: Sender<InputEvent>) -> JoinHandle<()> {
    spawn(move || {
        loop{
            // println!("loop");
            std::thread::sleep(std::time::Duration::from_secs(1));
            // Connect to socket
            let stream = match UnixStream::connect(SOCKET_PATH) {
                Err(_) => continue,
                Ok(stream) => stream,
            };
            stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();
    
            let mut reader = BufReader::new(&stream);
    
            loop {
                let mut my_str = String::new();
                match reader.read_line(&mut my_str) {
                    Ok(0) => {
                        // info!("Socket closed");
                        break;
                    }
                    Ok(_) => {
                        my_str = my_str.trim().to_string();
                        match my_str.len() {
                            0 => std::thread::sleep(std::time::Duration::from_millis(50)),
                            _ => {
                                let evt = parse_event(&my_str);
                                match evt.event_type {
                                    EvtType::ButtonPress => {
                                        sender.send(InputEvent::SetButtonState(true)).unwrap();
                                    }
                                    EvtType::ButtonRelease => {
                                        sender.send(InputEvent::SetButtonState(false)).unwrap();
                                    }
                                _ => {}
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        continue; // no data, but socket alive
                    }
                    Err(e) => {
                        // warn!("Error reading from socket {}",e);
                        break;
                    }
                }
            }
        }
    })
}

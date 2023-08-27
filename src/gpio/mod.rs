
mod cringedSocket;

use std::sync::mpsc::Sender;
use crate::state::InputEvent;
use std::thread::JoinHandle;


pub fn spawn_gpio(sender: Sender<InputEvent>) -> JoinHandle<()> {
    return cringedSocket::spawn_gpio(sender);
}

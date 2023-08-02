
#[cfg(raspi)]
mod raspi;
mod cringedSocket;

use std::sync::mpsc::Sender;
use crate::state::InputEvent;
use std::thread::JoinHandle;


pub fn spawn_gpio(sender: Sender<InputEvent>) -> JoinHandle<()> {
    #[cfg(raspi)]
    return raspi::spawn_gpio(sender);

    #[cfg(not(raspi))]
    return cringedSocket::spawn_gpio(sender);
}

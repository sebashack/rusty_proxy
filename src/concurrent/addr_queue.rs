use log::{error, info};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

type Addr = String;
type Port = u16;

pub struct AddrQueue {
    pub pusher: Sender<(Addr, Port)>,
    pub poller: Arc<Mutex<Receiver<(Addr, Port)>>>,
}

impl AddrQueue {
    pub fn new(addrs: Vec<(Addr, Port)>) -> Self {
        let (pusher, receiver) = mpsc::channel();

        for addr in addrs {
            pusher.send(addr);
        }

        let poller = Arc::new(Mutex::new(receiver));
        AddrQueue { pusher, poller }
    }

    pub fn clone(&self) -> Self {
        AddrQueue {
            pusher: self.pusher.clone(),
            poller: Arc::clone(&self.poller),
        }
    }
}

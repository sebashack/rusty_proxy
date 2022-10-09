use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc, Mutex,
};

pub struct CCFifoQueue<T> {
    pub pusher: SyncSender<T>,
    pub poller: Arc<Mutex<Receiver<T>>>,
}

impl<T> CCFifoQueue<T> {
    pub fn new(events: Vec<T>) -> Self {
        let (pusher, receiver) = mpsc::sync_channel(events.len());

        for evt in events {
            pusher.send(evt).unwrap();
        }

        let poller = Arc::new(Mutex::new(receiver));
        CCFifoQueue { pusher, poller }
    }

    pub fn clone(&self) -> Self {
        CCFifoQueue {
            pusher: self.pusher.clone(),
            poller: Arc::clone(&self.poller),
        }
    }
}

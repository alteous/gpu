use crossbeam_channel as chan;

const MAX_QUEUE_SIZE: usize = 1024;

/// Send half of a [`Queue`].
///
/// [`Queue`]: struct.Queue.html
pub type Sender<T> = chan::Sender<T>;

/// Receive half of a [`Queue`].
///
/// [`Queue`]: struct.Queue.html
pub type Receiver<T> = chan::Receiver<T>;

/// A thread-safe queue.
#[derive(Clone)]
pub struct Queue<T> {
    /// Send half of the queue.
    tx: Sender<T>,

    /// Receive half of the queue.
    rx: Receiver<T>,
}

impl<T> Queue<T> {
    /// Constructor.
    pub fn new() -> Self {
        let (tx, rx) = chan::bounded(MAX_QUEUE_SIZE);
        Self { tx, rx }
    }

    /// Clone the send half of the queue.
    pub fn tx(&self) -> Sender<T> {
        self.tx.clone()
    }

    /// Remove the item from the front of the queue.
    #[allow(dead_code)]
    pub fn next(&self) -> Option<T> {
        self.rx.try_recv().ok()
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

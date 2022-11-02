use crate::Message;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub trait MessageManager<T> {
    /// Start a manager loop which will send global messages and handle messages of own type
    fn start(&self, rx: Receiver<T>, tx: Sender<Message>) -> anyhow::Result<()>;
}

use crate::TypeInfo;
use codec::{Codec, Decode, Encode, EncodeLike};
use sp_std::prelude::*;

pub mod ringbuffer;

pub trait Queue<Item>
where
    Item: Codec + EncodeLike,
{
    /// Push an item onto the end of the queue.
    fn push(&mut self, i: Item);
    /// Pop an item from the start of the queue.
    fn pop(&mut self) -> Option<Item>;
    /// Return whether the queue is empty.
    fn is_empty(&self) -> bool;
}

pub trait Instantiable {
    type Args;
    fn new(args: Self::Args) -> Self;
}

#[derive(Clone, Eq, PartialEq, Default, Encode, Decode, TypeInfo, Debug)]
pub enum QueueSignal {
    #[default]
    PendingResponse,
    XcmSendError,
    ResponseReceived,
}

pub trait Queueable<Item>
where
    Item: Codec + EncodeLike,
{
    /// Push an item onto the end of the queue.
    fn push(i: Item);
    /// Pop an item from the start of the queue.
    fn pop() -> Option<Item>;
    /// Return whether the queue is empty.
    fn is_empty() -> bool;
}

/// This has some interesting functionality, since the ringbuffer is inherently based of drop & new.
impl<Item, R> Queueable<Item> for R
where
    Item: Codec + EncodeLike,
    R: Instantiable<Args = ()> + Queue<Item>,
{
    fn push(i: Item) {
        let mut r = R::new(());
        r.push(i);
    }

    fn pop() -> Option<Item> {
        let mut r = R::new(());
        r.pop()
    }

    fn is_empty() -> bool {
        let r = R::new(());
        r.is_empty()
    }
}

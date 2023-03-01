#![feature(box_syntax)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod receiver;
pub mod sender;

pub use receiver::Receiver;
pub use sender::Sender;

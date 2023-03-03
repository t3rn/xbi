use crate::{mock::*, xbi_abi::AccountId32, BufferRange, Error, Pallet, QueueItems, XbiResponses};
use crate::{Queue, H256};
use frame_support::{assert_err, assert_ok};
use xp_channel::traits::Writable;
use xp_channel::Message;
use xp_channel::XbiResult;
use xp_channel::{queue::Queue as QueueExt, XbiMetadata};
use xp_format::Timestamp;
use xp_format::XbiFormat;
use xs_channel::Receiver as ReceiverExt;
use xs_channel::Sender as SenderExt;

macro_rules! get_len {
    () => {{
        fn get_len() -> u16 {
            let (start, end) = <BufferRange<Test>>::get();
            (end - start)
        }
        get_len()
    }};
}

#[test]
fn cannot_store_duplicate_responses() {
    new_test_ext().execute_with(|| {
        let result = XbiResult::default();
        let hash = H256::from_low_u64_be(1);
        assert_ok!(XbiPortal::write((hash, result.clone())));
        assert_err!(
            XbiPortal::write((hash, result.clone())),
            Error::<Test>::ResponseAlreadyStored
        );
        let stored = XbiResponses::<Test>::get(hash).unwrap();
        assert_eq!(stored, result);
    });
}

#[test]
fn assert_nonce_incremented_and_id_enriched() {}

#[test]
fn test_async_sender_pushes_request_to_queue() {
    new_test_ext().execute_with(|| {
        let mut format = XbiFormat {
            instr: xp_format::XbiInstruction::Transfer {
                dest: AccountId32::new([4u8; 32]),
                value: 100,
            },
            ..Default::default()
        };
        assert_ok!(crate::pallet::AsyncSender::<Test>::send(Message::Request(
            format.clone()
        )));
        format.metadata.progress(Timestamp::Submitted(0));

        let mut queue = <Queue<Pallet<Test>>>::default();

        assert!(!queue.is_empty());
        assert_eq!(get_len!(), 1);

        let (msg, signal) = queue.pop().unwrap();
        assert_eq!(msg, Message::Request(format));
        assert_eq!(signal, xp_channel::queue::QueueSignal::PendingRequest);
    });
}

#[test]
fn test_async_sender_pushes_response_to_queue() {
    new_test_ext().execute_with(|| {
        let result = XbiResult::default();
        let mut meta = XbiMetadata::default();

        assert_ok!(crate::pallet::AsyncSender::<Test>::send(Message::Response(
            result.clone(),
            meta.clone()
        )));

        let mut queue = <Queue<Pallet<Test>>>::default();
        assert!(!queue.is_empty());
        assert_eq!(get_len!(), 1);

        let (msg, signal) = queue.pop().unwrap();
        meta.progress(Timestamp::Responded(0));
        assert_eq!(msg, Message::Response(result, meta));
        assert_eq!(signal, xp_channel::queue::QueueSignal::PendingResponse);
    });
}

#[test]
fn test_async_receiver_pushes_execution_to_queue() {
    new_test_ext().execute_with(|| {
        let mut format = XbiFormat {
            instr: xp_format::XbiInstruction::Transfer {
                dest: AccountId32::new([4u8; 32]),
                value: 100,
            },
            ..Default::default()
        };
        assert_ok!(crate::pallet::AsyncReceiver::<Test>::handle_request(
            &<Test as frame_system::Config>::Origin::root(),
            &mut format.clone()
        ));
        format.metadata.progress(Timestamp::Delivered(0));

        let mut queue = <Queue<Pallet<Test>>>::default();

        assert!(!queue.is_empty());
        assert_eq!(get_len!(), 1);

        let (msg, signal) = queue.pop().unwrap();
        assert_eq!(msg, Message::Request(format));
        assert_eq!(signal, xp_channel::queue::QueueSignal::PendingExecution);
    });
}

#[test]
fn test_async_receiver_pushes_result_to_queue() {
    new_test_ext().execute_with(|| {
        let mut metadata = XbiMetadata::default();
        let result = XbiResult::default();

        assert_ok!(crate::pallet::AsyncReceiver::<Test>::handle_response(
            &<Test as frame_system::Config>::Origin::root(),
            &result,
            &metadata
        ));
        metadata.progress(Timestamp::Received(0));

        let mut queue = <Queue<Pallet<Test>>>::default();

        assert!(!queue.is_empty());
        assert_eq!(get_len!(), 1);

        let (msg, signal) = queue.pop().unwrap();
        assert_eq!(msg, Message::Response(result, metadata));
        assert_eq!(signal, xp_channel::queue::QueueSignal::PendingResult);
    });
}

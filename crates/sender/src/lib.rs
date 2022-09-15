use sp_runtime::traits::Dispatchable;
use sp_runtime::DispatchResultWithInfo;

/// Sender is the trait providing the base `send` function.
/// All implementations of subtypes of sender must also provide an implementation of `send`.
/// The subtype of send would then utilise this trait to send a message, and then register the handler/promise
/// to handle the result.
trait Sender<T> {
    type Outcome;
    fn send(req: T) -> Self::Outcome;
}

/// ResolveSender is a handler that allows the user to specify what to do with the response
/// This allows for extensible middleware that could possibly end in a promise
trait ResolveSender<T>: Sender<T> {
    fn resolve<F: FnOnce(Self::Outcome) -> Self::Outcome>(req: T, f: Box<F>) -> Self::Outcome;
}

trait Promise {
    type Result;
}

// TODO: this could only be implemented in frame with `Call`
// #[cfg(feature = "frame")]
pub struct CallPromise<Result, Call: Dispatchable>(pub fn(Result) -> Call);

// Xbi promise delegates are defined as components that may handle the result of a sent message
// this would allow for chaining of middleware, I think
trait PromiseDelegate<T, Call: Dispatchable>: Sender<T> {
    fn then(
        req: T,
        promise: CallPromise<Self::Outcome, Call>,
    ) -> DispatchResultWithInfo<Call::PostInfo>;

    fn join(req: Vec<T>, promise: CallPromise<Vec<Self::Outcome>, Call>);

    fn chain(
        result: Call::PostInfo,
        req: T,
        promise: CallPromise<Self::Outcome, Call>,
    ) -> DispatchResultWithInfo<Call::PostInfo>;
}

// Note: because of the store simulation, if one test fails, they all will fail. Run each one independently to find the busted test.
#[cfg(test)]
mod tests {
    use super::*;

    use sp_runtime::DispatchResultWithInfo;
    use std::collections::HashMap;

    // Simulate some storage
    lazy_static::lazy_static! {
        static ref QUEUE: std::sync::Mutex<HashMap<u8, u32>> = {
            let m = HashMap::new();
            std::sync::Mutex::new(m)
        };
        static ref DISPATCH_RESULTS: std::sync::Mutex<HashMap<u64, u8>> = {
            let m = HashMap::new();
            std::sync::Mutex::new(m)
        };
    }

    #[derive(Debug, Clone)]
    enum TestError {}

    // Mock dispatchable that simply updates the calls with a byte
    struct DummyDispatch(u8);

    impl Dispatchable for DummyDispatch {
        type Config = Vec<u8>;
        type Info = Vec<u8>;
        type Origin = u64;
        type PostInfo = u8;
        fn dispatch(self, origin: Self::Origin) -> DispatchResultWithInfo<Self::PostInfo> {
            let mut guard = DISPATCH_RESULTS.lock().unwrap();
            guard.insert(origin, self.0);

            Ok(1)
        }
    }

    struct DummySender {}

    impl Sender<u32> for DummySender {
        type Outcome = Result<u32, TestError>;
        fn send(req: u32) -> Self::Outcome {
            let mut guard = QUEUE.lock().unwrap();
            guard.insert(1_u8, req);
            Ok(req)
        }
    }

    impl ResolveSender<u32> for DummySender {
        fn resolve<F: FnOnce(Self::Outcome) -> Self::Outcome>(
            req: u32,
            f: Box<F>,
        ) -> Self::Outcome {
            f(DummySender::send(req))
        }
    }

    const DUMMY_ORIGIN: u64 = 50;

    impl PromiseDelegate<u32, DummyDispatch> for DummySender {
        fn then(
            req: u32,
            promise: CallPromise<Self::Outcome, DummyDispatch>,
        ) -> DispatchResultWithInfo<<DummyDispatch as Dispatchable>::PostInfo> {
            promise.0(DummySender::send(req)).dispatch(DUMMY_ORIGIN)
        }

        fn join(req: Vec<u32>, promise: CallPromise<Vec<Self::Outcome>, DummyDispatch>) {
            promise.0(req.iter().map(|req| DummySender::send(*req)).collect())
                .dispatch(DUMMY_ORIGIN)
                .unwrap();
        }

        fn chain(
            result: <DummyDispatch as Dispatchable>::PostInfo,
            req: u32,
            promise: CallPromise<Self::Outcome, DummyDispatch>,
        ) -> DispatchResultWithInfo<<DummyDispatch as Dispatchable>::PostInfo> {
            let is_even = req % 2 == 0;
            if is_even {
                promise.0(DummySender::send(req)).dispatch(DUMMY_ORIGIN)
            } else {
                Ok(result)
            }
        }
    }

    #[serial_test::serial]
    #[test]
    fn sender_updates_queue() {
        DummySender::send(500).unwrap();
        let guard = QUEUE.try_lock().unwrap();
        assert_eq!(*guard.get(&1_u8).unwrap(), 500)
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_resolver_updates_queue() {
        DummySender::resolve(
            500,
            Box::new(|result| match result {
                Ok(x) => {
                    QUEUE.lock().unwrap().insert(1_u8, x + x);
                    assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 1000);

                    Ok(x)
                }
                Err(e) => {
                    QUEUE.lock().unwrap().insert(1_u8, 0);
                    Err(e)
                }
            }),
        )
        .unwrap();

        let guard = QUEUE.lock().unwrap();
        assert_eq!(*guard.get(&1_u8).unwrap(), 1000)
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_nested_resolver_updates_queue() {
        DummySender::resolve(
            500,
            Box::new(|result| {
                match result {
                    Ok(x) => {
                        // transform x
                        let new_x = x + x;
                        // update shared store with new x
                        QUEUE.lock().unwrap().insert(1_u8, new_x);
                        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 1000); // x + x
                        DummySender::resolve(new_x + new_x, Box::new(|result| result))
                    }
                    Err(e) => {
                        QUEUE.lock().unwrap().insert(1_u8, 0);
                        Err(e)
                    }
                }
            }),
        )
        .unwrap();

        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 2000)
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_super_nested_resolver_updates_queue() {
        DummySender::resolve(
            500,
            Box::new(|result: Result<u32, TestError>| {
                result
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
                    .and_then(|r| DummySender::resolve(r + r, Box::new(|result| result)))
            }),
        )
        .unwrap();

        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 65536000)
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_dispatch_updates_queue() {
        fn check_result_is_ok(result: Result<u32, TestError>) -> DummyDispatch {
            match result {
                Ok(_x) => DummyDispatch(1),
                Err(_e) => DummyDispatch(0),
            }
        }
        DummySender::then(500, CallPromise(check_result_is_ok)).unwrap();

        let guard = QUEUE.lock().unwrap();
        assert_eq!(*guard.get(&1_u8).unwrap(), 500);

        let guard = DISPATCH_RESULTS.lock().unwrap();
        assert_eq!(*guard.get(&50).unwrap(), 1_u8);
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_chainable_dispatch_updates_queue() {
        fn check_result_is_ok(result: Result<u32, TestError>) -> DummyDispatch {
            match result {
                Ok(_x) => DummyDispatch(1),
                Err(_e) => DummyDispatch(0),
            }
        }

        DummySender::then(500, CallPromise(check_result_is_ok))
            .and_then(|x| DummySender::chain(x, 100, CallPromise(check_result_is_ok)))
            .and_then(|x| DummySender::chain(x, 101, CallPromise(check_result_is_ok)))
            .and_then(|x| DummySender::chain(x, 102, CallPromise(check_result_is_ok)))
            .and_then(|x| DummySender::chain(x, 103, CallPromise(check_result_is_ok)))
            .unwrap();

        let guard = QUEUE.lock().unwrap();
        // is 102 due to the final chain condition not passing, proving the promise didnt update
        assert_eq!(*guard.get(&1_u8).unwrap(), 102);

        let guard = DISPATCH_RESULTS.lock().unwrap();
        assert_eq!(*guard.get(&50).unwrap(), 1_u8);
    }

    #[serial_test::serial]
    #[test]
    fn sender_with_set_of_dispatch_updates_queue() {
        fn check_result_is_ok(result: Vec<Result<u32, TestError>>) -> DummyDispatch {
            if result.iter().any(|x| x.is_err()) {
                DummyDispatch(0)
            } else {
                DummyDispatch(1)
            }
        }

        let mut requests: Vec<u32> = vec![];
        requests.resize(500, 1);
        let requests = requests
            .iter()
            .enumerate()
            .map(|(index, _)| (index + 1) as u32)
            .collect();

        DummySender::join(requests, CallPromise(check_result_is_ok));

        let guard = QUEUE.lock().unwrap();
        assert_eq!(*guard.get(&1_u8).unwrap(), 500);

        let guard = DISPATCH_RESULTS.lock().unwrap();
        assert_eq!(*guard.get(&50).unwrap(), 1_u8);
    }
}

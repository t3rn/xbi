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

// #[cfg(feature = "frame")] TODO: this could only be implemented in frame with `Call`
// TODO: implement promise
struct CallPromise<Call>(Call);

// Xbi promise delegates are defined as components that may handle the result of a sent message
// this would allow for chaining of middleware, I think
trait PromiseDelegate<T>: Sender<T> {
    fn then<F: Promise>(promise: F) -> Self::Outcome;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    // Simulate some storage
    lazy_static::lazy_static! {
        static ref QUEUE: std::sync::Mutex<HashMap<u8, u32>> = {
            let m = HashMap::new();
            std::sync::Mutex::new(m)
        };
    }

    #[derive(Debug, Clone)]
    enum TestError {}

    struct DummySender {}

    impl Sender<u32> for DummySender {
        type Outcome = Result<u32, TestError>;
        fn send(req: u32) -> Self::Outcome {
            QUEUE.lock().unwrap().insert(1_u8, req);
            // Simulate a result where
            Ok(req + req)
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

    #[test]
    fn sender_updates_queue() {
        DummySender::send(500).unwrap();
        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 500)
    }

    #[test]
    fn sender_with_resolver_updates_queue() {
        DummySender::resolve(
            500,
            Box::new(|result| match result {
                Ok(x) => {
                    QUEUE.lock().unwrap().insert(1_u8, x + x);
                    assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 2000);

                    Ok(x)
                }
                Err(e) => {
                    QUEUE.lock().unwrap().insert(1_u8, 0);
                    Err(e)
                }
            }),
        )
        .unwrap();

        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 2000)
    }

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
                        assert_eq!(*QUEUE.lock().unwrap().get(&1_u8).unwrap(), 2000);
                        DummySender::resolve(new_x, Box::new(|result| result))
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
}

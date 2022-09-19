/// Receivers either handle requests (checkins) or responses(checkouts)
enum Message {
    // #[cfg(feature = "requests")]
    CheckIn(CheckIn),
    // #[cfg(feature = "responses")]
    Checkout(CheckOut),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receiver() {}
}

/// provides some trait for storage
///
///
/// could use sp-trie
///
///is injected into the module that uses it, minimally sender, possible receiver

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

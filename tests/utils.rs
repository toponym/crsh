#[cfg(test)]
mod utils{
    #[macro_export]
    macro_rules! string_vec {
        ($($x:expr),*) => (vec![$($x.to_string()),*])
    }
    #[macro_export]
    macro_rules! reg_token {
    ($x:expr) => (Token::Regular($x.into()))
    }
}
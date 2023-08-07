// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectionType {
    Read,
    Write,
    Append
}

// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Regular(String),
    Expansion,
    SingleQuote,
    DoubleQuote,
    Assignment,
    Redirection(RedirectionType),
    Pipe,
    CommandSeparator,
    SubshellStart,
    SubshellEnd,
    Home,
    Background,
    EOF
}
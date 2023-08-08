// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Regular(String),
    Expansion,
    SingleQuote,
    DoubleQuote,
    Assignment,
    LRedirect, // <
    RRedirect, // >
    Pipe,
    CommandSeparator,
    SubshellStart,
    SubshellEnd,
    Home,
    Background,
    EOF
}
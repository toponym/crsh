#[derive(Debug, PartialEq)]
pub enum Node {
    Command(Vec<String>, Vec<Node>),
    Pipeline(Vec<Node>),
    RedirectAppend(String),
    RedirectWrite(String),
    RedirectRead(String),
}

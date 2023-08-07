#[derive(Debug, PartialEq)]
pub enum Node {
    Command(Vec<String>),
    Pipeline(Vec<Node>)
}
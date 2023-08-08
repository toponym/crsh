// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectType {
    Read,
    Write,
    Append
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Command(Vec<String>, Vec<Node>),
    Pipeline(Vec<Node>),
    Redirect(RedirectType, String)
}
#[cfg(test)]
mod tests {
    use crsh::Crsh;
    use crsh::Node;
    
    #[test]
    fn parse_simple() {
        let command = "ls -a -b\n";
        let expected = Node::Pipeline(vec!(Node::Command(vec!("ls", "-a", "-b"))));
        assert_eq!(expected, Crsh::parse(command));
    }

    #[test]
    fn parse_pipeline() {
        let command = "cat myfile | grep -r | wc\n";
        let cmd0 = Node::Command(vec!("cat", "myfile"));
        let cmd1 = Node::Command(vec!("grep", "-r"));
        let cmd2 = Node::Command(vec!("wc"));
        let expected = Node::Pipeline(vec!(cmd0, cmd1, cmd2));
        assert_eq!(expected, Crsh::parse(command));
    }
}
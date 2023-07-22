use std::process::{Command, Output};

pub struct Crsh {}

impl Crsh{
    pub fn parse(input: &str) -> Vec<&str>{
        let command = input.trim();
        command.split_whitespace().collect()
    }

    pub fn execute(tokens: Vec<&str>) -> Result<Output, &'static str>{
        let mut tok_iter = tokens.iter();
        let command =  match tok_iter.next() {
            Some(token) => token,
            None => {return Err("Empty command")}
        };
        let args = tok_iter;

        let child_result = Command::new(command)
            .args(args)
            .spawn();
        let child = match child_result {
            Ok(child) => child,
            Err(_) => {return Err("Failed spawning command")}
        };

        let output_result = child.wait_with_output();
        match output_result {
            Ok(output) => Ok(output),
            Err(_) => Err("Failed running command")
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::Crsh;
    
    #[test]
    fn parse_simple() {
        let command = "ls -a -b\n";
        let expected = vec!["ls", "-a", "-b"];
        assert_eq!(expected, Crsh::parse(command));
    }
}


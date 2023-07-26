use std::process::{Command, Output, exit};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::env::set_current_dir;
use std::str::FromStr;


pub struct Crsh {}

impl Crsh{
    pub fn parse(input: &str) -> Vec<&str>{
        let command = input.trim();
        command.split_whitespace().collect()
    }

    pub fn execute(mut tokens: Vec<&str>) -> Result<Output, &'static str>{
        if tokens.len() == 0{
            return Err("Empty command");
        }
        let command = tokens.remove(0);
        let args = tokens;
        match command {
            "cd" => Self::cd_command(&args),
            "exit" => Self::exit_command(&args),
            _ => Self::general_command(command, &args)
        }

    }

    fn new_empty_output(exit_code: i32) -> Output{
        Output {
            status: ExitStatusExt::from_raw(exit_code),
            stdout: vec!(),
            stderr: vec!()
        }
    }

    fn cd_command(args: &[&str]) -> Result<Output, &'static str>{
        if args.len() != 1{
            return Err("Too many arguments");
        }
        let new_dir = args[0];
        let absolute_new_dir = Path::new(new_dir);
        match set_current_dir(&absolute_new_dir) {
            Ok(_) => Ok(Self::new_empty_output(0)),
            Err(_) => Err("Failed changing directory")
        }
    }

    fn exit_command(args: &[&str]) -> Result<Output, &'static str>{
        let mut exit_code = 0;
        if args.len() > 1{
            return Err("Too many arguments");
        }
        if args.len() == 1{
            match i32::from_str(args[0]) {
                Ok(code) => exit_code = code,
                Err(_) => return Err("Didn't pass numeric argument")
            }
        }
        exit(exit_code);
    }


    fn general_command(command: &str, args: &[&str]) -> Result<Output, &'static str>
    {
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


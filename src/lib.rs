use std::process::{Command, Output, Child, exit, Stdio};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::env::set_current_dir;
use std::str::FromStr;
// TODO best way to handle namespaces?
pub mod scanner;
pub mod parser;
pub mod token;
pub mod ast;
use crate::ast::Node;

pub struct Crsh {}

impl Crsh{

    pub fn execute(node: Node) -> Result<Output, &'static str>{
        // TODO add better error handling + recovery
        match node {
            Node::Pipeline(commands) => Self::pipeline_command(commands),
            Node::Command(_toks, _) => {
                todo!("Add redirect");/*
                let child_res = Self::execute_command(&toks, Stdio::inherit(), Stdio::inherit());
                match child_res {
                    Ok(child_opt) => Self::collect_command_output(child_opt),
                    Err(_) => Err("Failed running command")
                }
                */
            },
            Node::Redirect(_,_) => {todo!("Add redirect");}  
        }
    }

    fn execute_command(tokens: &Vec<String>, redirect: &[Node], stdin: Stdio, stdout: Stdio) -> Result<Option<Child>, &'static str>{
        if tokens.is_empty() {
            return Err("Empty command");
        }
        let command = tokens[0].as_str();
        let args = &tokens[1..];
        // TODO add piping for builtins
        match command {
            "cd" => Self::cd_command(args),
            "exit" => Self::exit_command(args),
            _ => Self::general_command(command, args, stdin, stdout)
        }
    }

    fn collect_command_output(command_child: Option<Child>) -> Result<Output, &'static str>{
        match command_child {
            Some(child) => {
                let output_result = child.wait_with_output();
                match output_result {
                    Ok(output) => Ok(output),
                    Err(_) => Err("Failed running command")
                }
            },
            None => Ok(Self::new_empty_output(0))
        }
    }

    fn new_empty_output(exit_code: i32) -> Output{
        Output {
            status: ExitStatusExt::from_raw(exit_code),
            stdout: vec!(),
            stderr: vec!()
        }
    }

    fn cd_command(args: &[String]) -> Result<Option<Child>, &'static str>{
        if args.len() != 1{
            return Err("Too many arguments");
        }
        let new_dir = &args[0];
        let absolute_new_dir = Path::new(&new_dir);
        match set_current_dir(absolute_new_dir) {
            Ok(_) => Ok(None),
            Err(_) => Err("Failed changing directory")
        }
    }

    fn exit_command(args: &[String]) -> Result<Option<Child>, &'static str>{
        let mut exit_code = 0;
        if args.len() > 1{
            return Err("Too many arguments");
        }
        if args.len() == 1{
            match i32::from_str(&args[0]) {
                Ok(code) => exit_code = code,
                Err(_) => return Err("Didn't pass numeric argument")
            }
        }
        exit(exit_code);
    }


    fn general_command(command: &str, args: &[String], stdin: Stdio, stdout: Stdio) -> Result<Option<Child>, &'static str>
    {
        let child_result = Command::new(command)
            .args(args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn();
        match child_result {
            Ok(child) => Ok(Some(child)),
            Err(_) => Err("Failed spawning command")
        }
    }

    fn pipeline_command(commands: Vec<Node>) -> Result<Output, &'static str>{
        let mut previous_command = None;
        let command_count = commands.len();
        for (idx, command) in commands.iter().enumerate(){
            let stdin = previous_command
                .map_or(
                    Stdio::inherit(),
                    |child: Child| Stdio::from(child.stdout.unwrap())
                );
            let stdout = if idx < command_count-1 {
                Stdio::piped()
            } else {
                Stdio::inherit()
            };
            match command {
                Node::Command(toks, redirect) => {
                    match Self::execute_command(toks, redirect, stdin, stdout) {
                        Ok(child_opt) => {
                            previous_command = child_opt
                        },
                        Err(_) => {return Err("Pipeline command failed")}
                    }
                },
                _ => unimplemented!("Command {:?} not implemented for pipeline", command)
            }
        }
        if let Some(final_command) = previous_command {
            match final_command.wait_with_output() {
                Err(_) => Err("Pipeline command failed"),
                Ok(output) => Ok(output)
            }
        } else {
            Ok(Self::new_empty_output(0))
        }
    }
}

use std::env::set_current_dir;
use std::fs::OpenOptions;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::ExitStatus;
use std::process::{exit, Child, Command, Output, Stdio};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

// TODO best way to handle namespaces?
pub mod ast;
pub mod parser;
pub mod scanner;
pub mod token;
use crate::ast::Node;

pub struct Crsh {
    sigint_receiver: mpsc::Receiver<bool>,
}

impl Default for Crsh {
    fn default() -> Self {
        Self::new()
    }
}

impl Crsh {
    pub fn new() -> Self {
        let (sender, receiver): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
        ctrlc::set_handler(move || {
            sender.send(true).unwrap();
        })
        .expect("Error setting ctrl-c handler");
        Self {
            sigint_receiver: receiver,
        }
    }

    pub fn execute(&mut self, node: Node) -> Result<Output, &'static str> {
        // TODO add better error handling + recovery
        match node {
            Node::Pipeline(commands) => self.pipeline_command(commands),
            Node::CommandSequence(command_seq) => self.command_sequence(command_seq),
            _ => Err("Unexpected starting node"),
        }
    }

    fn command_sequence(&mut self, command_seq: Vec<Node>) -> Result<Output, &'static str> {
        let mut res = Ok(Self::new_empty_output(0));
        // TODO support command in command sequence
        for command in command_seq {
            res = match command {
                Node::Pipeline(commands) => self.pipeline_command(commands),
                _ => Err("Unexpected node in command sequence"),
            };
        }
        res
    }

    fn execute_command(
        tokens: &Vec<String>,
        redirects: &[Node],
        stdin: Stdio,
        stdout: Stdio,
    ) -> Result<Option<Child>, &'static str> {
        if tokens.is_empty() {
            return Err("Empty command");
        }
        let command = tokens[0].as_str();
        let args = &tokens[1..];
        let mut cmd_stdin = stdin;
        let mut cmd_stdout = stdout;
        // TODO add piping for builtins
        for redirect in redirects {
            match redirect {
                Node::RedirectRead(filename) => {
                    let file = OpenOptions::new()
                        .read(true)
                        .open(filename)
                        .map_err(|_| "Failed opening file")?;
                    cmd_stdin = Stdio::from(file);
                }
                Node::RedirectWrite(filename) => {
                    let file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(filename)
                        .map_err(|_| "Failed opening file")?;
                    cmd_stdout = Stdio::from(file);
                }
                Node::RedirectAppend(filename) => {
                    let file = OpenOptions::new()
                        .append(true)
                        .open(filename)
                        .map_err(|_| "Failed opening file")?;
                    cmd_stdout = Stdio::from(file);
                }
                _ => panic!("Unexpected node for redirect: {:?}", redirect),
            }
        }
        match command {
            "cd" => Self::cd_command(args),
            "exit" => Self::exit_command(args),
            _ => Self::general_command(command, args, cmd_stdin, cmd_stdout),
        }
    }

    fn new_empty_output(exit_code: i32) -> Output {
        Output {
            status: ExitStatusExt::from_raw(exit_code),
            stdout: vec![],
            stderr: vec![],
        }
    }

    fn cd_command(args: &[String]) -> Result<Option<Child>, &'static str> {
        if args.len() != 1 {
            return Err("Too many arguments");
        }
        let new_dir = &args[0];
        let absolute_new_dir = Path::new(&new_dir);
        match set_current_dir(absolute_new_dir) {
            Ok(_) => Ok(None),
            Err(_) => Err("Failed changing directory"),
        }
    }

    fn exit_command(args: &[String]) -> Result<Option<Child>, &'static str> {
        let mut exit_code = 0;
        if args.len() > 1 {
            return Err("Too many arguments");
        }
        if args.len() == 1 {
            match i32::from_str(&args[0]) {
                Ok(code) => exit_code = code,
                Err(_) => return Err("Didn't pass numeric argument"),
            }
        }
        println!("exit");
        exit(exit_code);
    }

    fn general_command(
        command: &str,
        args: &[String],
        stdin: Stdio,
        stdout: Stdio,
    ) -> Result<Option<Child>, &'static str> {
        let child_result = Command::new(command)
            .args(args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn();
        match child_result {
            Ok(child) => Ok(Some(child)),
            Err(_) => Err("Failed spawning command"),
        }
    }

    fn wait_handle_interrupts(&mut self, child: &mut Child) -> Result<ExitStatus, &'static str> {
        loop {
            match child.try_wait() {
                Ok(None) => (),                        // Child still running
                Ok(Some(status)) => return Ok(status), // Child finished
                Err(_) => return Err("Error waiting on process"),
            }
            if let Ok(true) = self.sigint_receiver.try_recv() {
                child.kill().expect("Error killing child process"); // sigint received, kill child
                println!("SIGINT RECEIVED");
                return Err("Error SIGINT received, child killed");
            }
            sleep(Duration::from_millis(100));
        }
    }

    fn pipeline_command(&mut self, commands: Vec<Node>) -> Result<Output, &'static str> {
        let mut previous_cmd: Option<Child> = None;
        let command_count = commands.len();
        for (idx, command) in commands.iter().enumerate() {
            let stdin = previous_cmd.map_or(Stdio::inherit(), |mut child: Child| {
                Stdio::from(child.stdout.take().unwrap())
            });
            let stdout = if idx < command_count - 1 {
                Stdio::piped()
            } else {
                Stdio::inherit()
            };
            let mut current_cmd = match command {
                Node::Command(toks, redirect) => {
                    match Self::execute_command(toks, redirect, stdin, stdout) {
                        Ok(child_opt) => child_opt,
                        Err(err) => return Err(err),
                    }
                }
                _ => unimplemented!("Command {:?} not implemented for pipeline", command),
            };
            if let Some(child) = current_cmd.as_mut() {
                self.wait_handle_interrupts(child)?;
            }
            previous_cmd = current_cmd;
        }
        match previous_cmd {
            Some(final_command) => final_command
                .wait_with_output()
                .map_err(|_| "Error waiting for output"),
            None => Ok(Self::new_empty_output(0)),
        }
    }
}

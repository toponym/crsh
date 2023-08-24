use std::env::set_current_dir;
use std::fmt::Display;
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
#[derive(Debug)]
enum InterpretErr {
    RuntimeError(&'static str),
    Interrupt(&'static str),
    ExitStatusFailure(&'static str), // for crsh builtins
}

impl Display for InterpretErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
            Self::Interrupt(msg) => write!(f, "Interrupt: {}", msg),
            Self::ExitStatusFailure(msg) => write!(f, "ExitStatusFailure: {}", msg),
        }
    }
}

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

    pub fn execute(&mut self, node: Node) -> Result<Output, String> {
        // TODO catch interrupt error here
        self.clear_handler();
        match node {
            Node::Pipeline(commands) => self
                .pipeline_command(commands)
                .map_err(|err| format!("{}", err)),
            Node::CommandSequence(command_seq) => self
                .command_sequence(command_seq)
                .map_err(|err| format!("{}", err)),
            _ => Err("Unexpected starting node".to_string()),
        }
    }

    fn clear_handler(&mut self) {
        // clear the channel with the sigint handler
        while self.sigint_receiver.try_recv().is_ok() {}
    }

    fn command_sequence(&mut self, command_seq: Vec<Node>) -> Result<Output, InterpretErr> {
        let mut res = Ok(Self::new_empty_output(0));
        // TODO support command in command sequence
        for command in command_seq {
            res = match command {
                Node::Pipeline(commands) => match self.pipeline_command(commands) {
                    Ok(output) => Ok(output),
                    Err(InterpretErr::ExitStatusFailure(_)) => Ok(Self::new_empty_output(1)),
                    Err(InterpretErr::Interrupt(_)) => Ok(Self::new_empty_output(130)),
                    Err(InterpretErr::RuntimeError(msg)) => {
                        return Err(InterpretErr::RuntimeError(msg))
                    }
                },
                _ => Err(InterpretErr::RuntimeError(
                    "Unexpected node in command sequence",
                )),
            };
        }
        res
    }

    fn execute_command(
        tokens: &Vec<String>,
        redirects: &[Node],
        stdin: Stdio,
        stdout: Stdio,
    ) -> Result<Option<Child>, InterpretErr> {
        if tokens.is_empty() {
            return Err(InterpretErr::RuntimeError("Empty command"));
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
                        .map_err(|_| InterpretErr::RuntimeError("Failed opening file"))?;
                    cmd_stdin = Stdio::from(file);
                }
                Node::RedirectWrite(filename) => {
                    let file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(filename)
                        .map_err(|_| InterpretErr::RuntimeError("Failed opening file"))?;
                    cmd_stdout = Stdio::from(file);
                }
                Node::RedirectAppend(filename) => {
                    let file = OpenOptions::new()
                        .append(true)
                        .open(filename)
                        .map_err(|_| InterpretErr::RuntimeError("Failed opening file"))?;
                    cmd_stdout = Stdio::from(file);
                }
                _ => panic!("Unexpected node for redirect: {:?}", redirect),
            }
        }
        let res = match command {
            "cd" => Self::cd_command(args),
            "exit" => Self::exit_command(args),
            _ => Self::general_command(command, args, cmd_stdin, cmd_stdout),
        };
        if let Err(InterpretErr::ExitStatusFailure(_)) = res {
            Ok(None)
        } else {
            res
        }
    }

    fn new_empty_output(exit_code: i32) -> Output {
        Output {
            status: ExitStatusExt::from_raw(exit_code),
            stdout: vec![],
            stderr: vec![],
        }
    }

    fn cd_command(args: &[String]) -> Result<Option<Child>, InterpretErr> {
        if args.len() != 1 {
            println!("Too many directories");
            return Err(InterpretErr::ExitStatusFailure(""));
        }
        let new_dir = &args[0];
        let absolute_new_dir = Path::new(&new_dir);
        match set_current_dir(absolute_new_dir) {
            Ok(_) => Ok(None),
            Err(_) => {
                println!("Failed changing directory");
                Err(InterpretErr::ExitStatusFailure(""))
            }
        }
    }

    fn exit_command(args: &[String]) -> Result<Option<Child>, InterpretErr> {
        let mut exit_code = 0;
        if args.len() > 1 {
            println!("Too many arguments");
            return Err(InterpretErr::ExitStatusFailure(""));
        }
        if args.len() == 1 {
            match i32::from_str(&args[0]) {
                Ok(code) => exit_code = code,
                Err(_) => {
                    println!("Didn't pass numeric argument");
                    return Err(InterpretErr::ExitStatusFailure(""));
                }
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
    ) -> Result<Option<Child>, InterpretErr> {
        let child_result = Command::new(command)
            .args(args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn();
        match child_result {
            Ok(child) => Ok(Some(child)),
            Err(_) => Err(InterpretErr::RuntimeError("Failed spawning command")),
        }
    }

    fn wait_handle_interrupts(&mut self, child: &mut Child) -> Result<ExitStatus, InterpretErr> {
        loop {
            match child.try_wait() {
                Ok(None) => (),                        // Child still running
                Ok(Some(status)) => return Ok(status), // Child finished
                Err(_) => return Err(InterpretErr::RuntimeError("Error waiting on process")),
            }
            if let Ok(true) = self.sigint_receiver.try_recv() {
                child.kill().expect("Error killing child process"); // sigint received, kill child
                return Err(InterpretErr::Interrupt("SIGINT Received"));
            }
            sleep(Duration::from_millis(100));
        }
    }

    fn pipeline_command(&mut self, commands: Vec<Node>) -> Result<Output, InterpretErr> {
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
                .map_err(|_| InterpretErr::RuntimeError("Error waiting for output")),
            None => Ok(Self::new_empty_output(0)),
        }
    }
}

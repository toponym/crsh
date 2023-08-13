use std::env::set_current_dir;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{exit, Child, Command, Output, Stdio};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// TODO best way to handle namespaces?
pub mod ast;
pub mod parser;
pub mod scanner;
pub mod token;
use crate::ast::Node;

pub struct Crsh {
    previous_cmd: Arc<Mutex<Option<Child>>>,
}

impl Default for Crsh {
    fn default() -> Self {
        Self::new()
    }
}

impl Crsh {
    pub fn new() -> Self {
        let previous_cmd: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
        Self { previous_cmd }
    }

    pub fn set_ctrl_handler(&mut self) {
        // set SIGINT handler
        let previous_command_clone = self.previous_cmd.clone();
        ctrlc::set_handler(move || {
            // TODO don't use unwrap here?
            if let Some(child) = previous_command_clone.lock().unwrap().as_mut() {
                child.kill().expect("Error killing child process");
            }
        })
        .expect("Error setting ctrl-c handler");
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

    fn ref_wait_with_output(child: &mut Child) -> Result<Output, &'static str> {
        /*
        Similar to std::process::Child::wait_with_output
        Differences are this function does not take stdin (which supposedly prevents deadlock)
        and this function does not take ownership of child
        */
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();

        macro_rules! output_to_vec {
            ($vec:expr, $child_output: expr, $err_msg:expr) => {
                match $child_output.take() {
                    Some(mut child_out) => match child_out.read_to_end(&mut $vec) {
                        Ok(_) => (),
                        Err(_) => {
                            return Err($err_msg);
                        }
                    },
                    None => (),
                }
            };
        }
        output_to_vec!(stdout, child.stdout, "Error reading command stdout");
        output_to_vec!(stderr, child.stderr, "Error reading command stderr");

        let status = child.wait().map_err(|_| "Failed running final command")?;

        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }

    fn pipeline_command(&mut self, commands: Vec<Node>) -> Result<Output, &'static str> {
        self.previous_cmd = Arc::new(Mutex::new(None));
        let command_count = commands.len();
        for (idx, command) in commands.iter().enumerate() {
            let stdin = self
                .previous_cmd
                .lock()
                .unwrap()
                .as_mut()
                .map_or(Stdio::inherit(), |child: &mut Child| {
                    Stdio::from(child.stdout.take().unwrap())
                });
            let stdout = if idx < command_count - 1 {
                Stdio::piped()
            } else {
                Stdio::inherit()
            };
            match command {
                Node::Command(toks, redirect) => {
                    match Self::execute_command(toks, redirect, stdin, stdout) {
                        Ok(child_opt) => *self.previous_cmd.lock().unwrap() = child_opt,
                        Err(err) => return Err(err),
                    }
                }
                _ => unimplemented!("Command {:?} not implemented for pipeline", command),
            }
        }
        match self.previous_cmd.lock().unwrap().as_mut() {
            Some(final_command) => Self::ref_wait_with_output(final_command),
            None => Ok(Self::new_empty_output(0)),
        }
    }
}

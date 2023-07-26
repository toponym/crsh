# crsh
crsh: A crab shell in Rust


## TODO
- piping between commands
- redirect stdin/stdout/stderr to/from files
- signal management (e.g. SIGINT w ctrl+c)
- environment variables (`export`, `unset`)
- shell builtins: `history`
- EOF (ctrl + D)
- quoting
- `&&`, `||`, `;`

## References
- [Build Your Own Shell using Rust](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/)
- [I/O Redirection in the Shell](https://thoughtbot.com/blog/input-output-redirection-in-the-shell)
- [Bash Manual Basic Shell Features](https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features)
- [Bash Manual Shell Builtin Commands](https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html)
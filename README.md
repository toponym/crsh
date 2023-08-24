# crsh
crsh: A crab shell in Rust


## Features
- Pipes
- Shell builtins: `cd`, `exit`
- Redirect stdin/stdout
- handle SIGINT from ctrl+c
- handle EOF (ctrl+D)
- command sequences with `;`

## TODO
- redirect stderr to/from files
- environment variables (`export`, `unset`)
- shell builtins: `history`
- quoting
- `&&`, `||`
- graceful scanning and parsing error handling

## EBNF Grammar
```
command_sequence ::= pipeline {";" pipeline} [";"];
pipeline ::= command {"|" command }
command ::= word {word} {redirect}
word ::= regular_char {regular_char}
redirect ::= '>' word
              | '<' word
              | '>>' word

```
- A `regular_char` is a character that is not a Bash special character (`"$'\"\\#=[]!><|;{}()*?~&`). This isn't proper EBNF, but I chose to leave it like this for simplicity.
- For the subset I support, I make some assumptions about the grammar to make my life easier.
## References
- [Build Your Own Shell using Rust](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/)
- [I/O Redirection in the Shell](https://thoughtbot.com/blog/input-output-redirection-in-the-shell)
- [Bash Manual Basic Shell Features](https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features)
- [Bash Manual Shell Builtin Commands](https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html)
- [Bash Special Characters](https://mywiki.wooledge.org/BashGuide/SpecialCharacters)
- [Crafting Interpreters](https://craftinginterpreters.com/)

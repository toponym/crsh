# crsh
crsh: A crab shell in Rust

This is a small bash-esque shell I implemented to learn more about Rust.

## Features
- Pipes
- Shell builtins: `cd`, `exit`
- Redirect stdin/stdout
- Handle SIGINT from ctrl+c
- Handle EOF (ctrl+D)
- Command sequences with `;`
- Quotes


## EBNF Grammar
```
command_sequence ::= pipeline {";" pipeline} {";"};
pipeline ::= command {"|" command }
command ::= word {word} {redirect}
word ::= regular_word 
        | quoted_word
regular_word ::= regular_char {regular_char}
quoted_word ::= single_quoted_word 
        | double_quoted_word
single_quoted_word ::= "'" not_single_quote {not_single_quote} "'"
double_quoted_word ::= """ not_double_quote {not_double_quote} """
redirect ::= '>' word
        | '<' word
        | '>>' word

```
- A `regular_char` is a character that is not a Bash special character (`"$'\"\\#=[]!><|;{}()*?~&`). This isn't proper EBNF, but I chose to leave it like this for simplicity.
    - Similarly, `not_single_quote` and `not_double_quote` are any character that is not `'` or `"`, respectively.
- For the subset I support, I make some assumptions about the grammar to make my life easier.
## References
- [Build Your Own Shell using Rust](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/)
- [I/O Redirection in the Shell](https://thoughtbot.com/blog/input-output-redirection-in-the-shell)
- [Bash Manual Basic Shell Features](https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features)
- [Bash Manual Shell Builtin Commands](https://www.gnu.org/software/bash/manual/html_node/Shell-Builtin-Commands.html)
- [Bash Special Characters](https://mywiki.wooledge.org/BashGuide/SpecialCharacters)
- [Crafting Interpreters](https://craftinginterpreters.com/)

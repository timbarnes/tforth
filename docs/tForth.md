# tForth - a minimal Forth in Rust
## by Tim Barnes

tForth is a simple implementation of some basic Forth language capabilities. Where possible, I have followed the [Forth standard:](https://forth-standard.org). My intent was to simultaneously learn Forth and Rust. 

The program relies on a Rust binary, and a source file containing library functions (currently quite small).
The program should be installed anywhere, but the default location for the library file is '~/.tforth/corelib.fs'.

## Command line Options
tForth responds to the following command line options:

| command line              |                                                                                          | notes |
| ------------------------- | ---------------------------------------------------------------------------------------- | ----- |
| Usage: `tforth [OPTIONS]` |                                                                                          |
| Options:                  |                                                                                          |
| `--debuglevel <VALUE>`    | [possible values: error, warning, info, debug]                                           |
| `--library <VALUE>`       | Allows a library other than the standard core library to be loaded at startup.           |
| `--file <VALUE>`          | Allows a user-defined tForth code file to be loaded after (or without) the library file. |
| `--nocore`                | Suppresses loading of a core / library file                                              |
| ` -h, --help`             | Print help                                                                               |
| `-V, --version`           | Print version'                                                                           |

  tForth is an interactive command-line program that can be used like a reverse-polish calculator. Operands (integers) are placed on the calculation stack. Operators consume and operate on stack elements. For example:

| code  | example                                                |
| ----- | ------------------------------------------------------ |
| `2`   | places `2` on the stack                                |
| `3 4` | places `3`, then `4` on the stack;                     |
| `*`   | multiplies `3` by `4`, leaving the result on the stack |
| `+`   | adds the top of the stack to `2` (2nd to top of stack) |
| `14`  | The result is left on the stack.                       |
| `.s`  | is a tForth word to print the contents of the stack.   |

 ## Built-in and library words

 Conventions are as follows:

 + `(` and `)` enclose comments, and are ignored.
 + It is common practice to provide a stack template as documentation in word definitions. The left side shows the required stack elements (essentially arguments to the word), and the right side (past the "`--`") shows the results left on the stack. For example `( m -- n n )` would mean that a single value is required on the stack, and two new values are left after execution of the word. 
 + Underflow results from too few arguments being available on the stack.

## Comments, Strings and Print words

 | word         | stack signature | usage                                                                                                                                                                                                                                             |
 | ------------ | --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
 | `(`          | `( -- )`        | The left paren starts a comment. It requires a following space, as Forth's parser is simplistic. `(` is a word in its own right. It's behavior is to consume text following, up to and including the closing paren. It does not affect the stack. |
 | `.`          | `( n -- )`      | Prints the top of the stack, dropping (deleting from the stack) the printed number.                                                                                                                                                               |
 | `." <text>"` | `( -- )`        | Prints \<text> to the terminal. Essentially a print statement for constant text.                                                                                                                                                                  |
 | `s" <text>`  | `( -- )`        | Stores \<text> in a special variable location, from which it can be used by other words.                                                                                                                                                          |
 | `.s" <text>` | `( -- ) `       | Prints the stored string to the terminal.                                                                                                                                                                                                         |
 | `emit`       | `( c -- )`      | Emit a single character to the output stream, using the top of the stack as an ASCII code. Undefined if the number on the stack is not in the correct range of printable ASCII character.                                                         |

 ## Arithmetic and logic

 In Forth, false is represented with a zero value, while all other numerical values are considered truthy. The standard library provides synonoms for true and false using the canonical false value of -1 (all bit on), and 0 for true.
 
 | word | stack signature  | usage                                                                                               |
 | ---- | ---------------- | --------------------------------------------------------------------------------------------------- |
 | `+`  | (m n -- m+n )    | Adds the top two numbers on the stack, leaving the sum on the stack                                 |
 | `-`  | `( m n -- m-n )` | Subtracts the top of the stack from the element below, leaving the result on the stack.             |
 | `*`  | `( m n -- m*n )` | Multiplies the top two stack elements, leaving the result on the stack.                             |
 | `/`  | `( m n -- m/n )` | Divides the 2nd item by the top item, leaving the result on the stack.                              |
 | `<`  | ( m n -- b )     | if m < n, place the true value (-1) on the stack.                                                   |
 | `>`  | (m n -- b )      | If m > n, then push true, otherwise push false                                                      |
 | `0=` | ( n -- b )       | If the number on the stack is zero, replace it with true (-1); otherwise replace it with false (0). |

 ## Stack manipulation 

 Forth is unashamedly a stack language, relying on the stack for almost everything. It is therefore necessary to be able to manipulate stack values in a range of different ways to support algorithmic needs. These are the basic stack manipulation supported by tForth.

 | word   | signature            | usage                                                                                                                                               |
 | ------ | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
 | `dup`  | `( n -- n n )`       | Push a duplicate copy of the number on top of the stack                                                                                             |
 | `drop` | `( n -- )`           | Remove (pop) the top element off the stack                                                                                                          |
 | `swap` | `( m n -- n m )`     | Reverse the order of the top two items on the stack                                                                                                 |
 | `over` | `(m n -- m n m )`    | Push a copy of the second-to-top stack item onto the stack                                                                                          |
 | `rot`  | `( l m n -- m n l )` | Bring the third item to the top, effectively rotating the top three elements. Repeating this operation three times will restore the original order. |
 | `2dup` | `( m n -- m n m n )` | Duplicate the top two items on the stack.                                                                                                           |
 
 ## Definitions, branches and conditional forms

 | word               | signature  | usage                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
 | ------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
 | `: <name>`         | `( -- )`   | Starts the definition of a word. \<name> is the name of the word. Followed by a series of Forth words, and ended with a `;`. If the word has been previously defined (in Forth), it will be redefined. Built-in words cannot (currently) be redefined. Typical usage might be as follows: `: square ( n -- n*n ) dup * ;` - defining the word `square` that squares the number on top of the stack. Note the inclusion of the stack signature - this is only a comment, and is not enforced, but it's good practice as Forth is otherwise not the most readable of languages. |
 | `;`                | `( -- )`   | Ends the definition of the current word, causing tForth to save the list of words making up the definition for future interpretation.                                                                                                                                                                                                                                                                                                                                                                                                                                         |
 | `if - else - then` | `( b -- )` | Tests the item on the top of the stack. If it is a true value, executes the code between `if` and `else`, otherwise executes the code between `else` and `then`. `if` statements can be nested.                                                                                                                                                                                                                                                                                                                                                                               |
 | `if - then`        | `( b -- )` | An `if` statement with no `else` clause.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
 | `endif`            |            | A synonym for then, for those more comfortable with `if - else - endif` syntax.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |

Other control structures are in the process of being implemented. Forth typically supports for loops and a variety of other mechanisms.

## Variables

 | word         | signature    | usage                                                                                                                      |
 | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------------------- |
 | `variable x` | `( -- a )`   | Creates a new variable called x (any unique combination of non-whitespace characters is acceptable for the variable name). |
 | `<variable>` | `( -- a )`   | Places the address of the named variable on the stack.                                                                     |
 | `@`          | `( a -- n )` | Places the value of the variable addressed by a on the stack.                                                              |
 | `!`          | `( n a -- )` | Stores the value n in the variable addressed by a.                                                                         |


## Debugging
tForth provides a couple of mechanisms for debugging: engine (built-in) messages, and a stepper, combined with functions to display some of the engine internals.

### Implementation Messages
The first is a window into the engine, which enables printing of informational and debug messages. The following words are supported:

| word          | signature   | usage                                                                                                                           |
| ------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `debuglevel`  | ` ( n -- )` | Sets debuglevel to a value between 0 and 3, popping the provided value off the stack. See shortcuts below for more information. |
| `quiet`       | ` ( -- )`   | Sets debug to 0 => Show errors only                                                                                             |
| `dbg-warning` | `( -- )`    | Sets debug to 1 => Show warning and error messages                                                                              |
| `dbg-info`    | `( -- )`    | Sets debug to 2 => show info, warning and error messages                                                                        |
| `dbg-debug`   | `( -- )`    | Sets debug to 3 => show debug, info, warning and error messages.                                                                |

## Stepper
A simple stepper function is provided, along with some words to show some internal values in the engine.

| word         | signature | usage                                                                                              |
| ------------ | --------- | -------------------------------------------------------------------------------------------------- |
| `show-stack` | `( -- )`  | Tells the engine to print out the current stack values after each line of interactive computation. |
| `words`      | `( -- )`  | Prints a list of all the Forth-defined (library and user-define) words.                            |
| `see <word>` | `( -- )`  | Prints the definition of the Forth-defined word \<word>.                                           |
| `see-all`    | `( -- )`  | Prints definitions of all the Forth-defined words.                                                 |
| `variables`  | `( -- )`  | Prints a list of all defined variables and their values.                                           |
| `step-on`    | `( -- )`  | Enables single-step mode.                                                                          |
| `step-off`   | `( -- )`  | Disables single-step mode.                                                                         |

The single stepper stops before executing each word, and waits for user input. The stepper steps into definitions, so all Forth-defined words are shown in full. By default, the stepper prints the word that's about to be executed, followed by a prompt "Step> ". The stepper operations do not affect the stack, so they don't have stack signatures. There are currently no ways to alter variables or the stack during computation: this is simply a visibility tool. 

Valid inputs are a carriage return, or a single character followed by a carriage return:

| Command | Action                                                        |
| ------- | ------------------------------------------------------------- |
| `<cr>`  | Move to the next word                                         |
| `s`     | Print the stack and move to the next word                     |
| `v`     | Print variable values and move to the next word               |
| `a`     | Print the stack and variable values and move to the next word |
| `c`     | Disable the single-stepper and move to the next word.         |

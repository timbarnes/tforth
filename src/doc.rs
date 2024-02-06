// Documentation strings for built-in functions.

use std::collections::HashMap;
use std::string::String;

// mut doc_strings: HashMap<String, String> = HashMap::new();

pub fn build_doc_strings() -> HashMap<String, String> {
    let mut doc_strings: HashMap<String, String> = HashMap::new();

    macro_rules! doc {
        ($name:expr, $doc:expr) => {
            doc_strings.insert($name.to_owned(), $doc.to_owned());
        };
    }

    doc!("+", "( j k -- j+k ) Push j+k on the stack");
    doc!("+", "( j k -- j+k ) Push j+k on the stack");
    doc!("-", "( j k -- j-k ) Push  -k on the stack");
    doc!("*", "( j k -- j*k ) Push j*k on the stack");
    doc!("/", "( j k -- j/k ) Push j/k on the stack");
    doc!("mod", "( j k -- j/k ) Push j%k on the stack");
    doc!("=", "( j k -- b ) If j == k push true else false");
    doc!("<", "( j k -- j/k ) If j < k push true else false");
    doc!("0<", "( j k -- j/k ) If j < 0 push true else false");
    doc!(
        "true",
        "( -- -1 ) Push the canonical true value on the stack."
    );
    doc!(
        "false",
        "( -- 0 ) Push the canonical false value on the stack."
    );
    doc!(
        ".",
        "( n -- ) Pop the top of the stack and print it, followed by a newline"
    );
    doc!(
        "..",
        "( n -- ) Pop the top of the stack and print it without a newline"
    );
    doc!(
        "flush",
        "( -- ) Flush the stdout buffer. Required if no newline has been issued."
    );
    doc!(".s", "( -- ) Print the contents of the calculation stack");
    doc!(".s\"", "( -- ) Print the saved string to stdout");
    doc!(
        "show-stack",
        "( -- )Enable automatic printing of the stack to the console after each line of input"
    );
    doc!(
        "hide-stack",
        "( -- ) Suppress automatically printing the stack to the console"
    );
    doc!(
        "emit",
        "( c -- ) Emit a single character from the stack to stdout"
    );
    doc!("clear", "( n.. -- ) Empty the calculation stack");
    doc!(
        "dup",
        "( n -- n n ) Duplicate the item on the top of the stack"
    );
    doc!("drop", "( n -- ) Discard the top item from the stack");
    doc!(
        "swap",
        "( m n -- n m ) Reverse the order of the top two items on the stack"
    );
    doc!(
        "over",
        "( m n -- m n m ) Push a copy of the second item on the stack on top"
    );
    doc!(
        "rotate",
        "( i j k -- j k i ) Move the third stack item to the top"
    );
    doc!(
        "and",
        "( a b -- a & b ) Pop a and b, returning the logical and"
    );
    doc!(
        "or",
        "( a b -- a | b ) Pop a and b, returning the logical or"
    );
    doc!(
        "variable",
        "usage: variable <name> - creates a new variable called <name>
         Subsequent use of <name> places its address on the stack."
    );
    doc!(
        "constant",
        "usage: constant <name> ( v -- ) - creates a new constant called <name>, 
         taking its value from the stack. 
         Use of the name places the value (not the address) on the stack. "
    );
    doc!(
        "@",
        "( addr -- value ) Replaces the address of a variable with its value"
    );
    doc!(
        "!",
        "( value address -- ) Stores value in the variable at address"
    );
    doc!(
        "\\",
        "Defines an in-line comment. All text from the \\ to the end of line 
        will be ignored."
    );
    doc!("abort", "Ends the execution of the current word");
    doc!(
        "words",
        "Prints a list of the currently defined words. Does not list builtins"
    );
    doc!(
        "seeall",
        "Prints the definitions of currently defined words. Does not list builtins."
    );
    doc!(
        "see",
        "Usage: see <word> - Prints the help string for a builtin.
          Prints the definition for a Forth-defined word."
    );
    doc!(
        "stack-depth",
        "( -- n ) Places a number representing the depth of the stack onto the stack"
    );
    doc!("step-on", "( -- ) Invokes the single-stepper");
    doc!("step-off", "( -- ) Disables the single-stepper");
    doc!(
        ":",
        "Enter compile mode. Subsequent words up to a ';' will be added to the definition"
    );
    doc!(";", "Exits compile mode and saves the definition");

    doc_strings
}

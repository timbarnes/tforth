/// Inner Interpreters
///
/// Core functions to execute specific types of objects
///
use crate::engine::TF;

impl TF {
    /// Executes the builtin at the next address in DATA
    ///
    ///    [ index of i_builtin ] [ index of builtin ] in a compiled word
    ///
    pub fn i_builtin(&mut self) {}

    /// Places the address of the adjacent variable on the stack
    ///
    ///    [ index of i_variable ] [ index of builtin ] in a compiled word
    ///
    pub fn i_variable(&mut self) {}

    /// Places the value of the adjacent constant on the stack
    ///
    ///    [ index of i_constant ] [ constant value ] in a compiled word
    ///
    pub fn i_constant(&mut self) {}

    /// Places the adjacent number on the stack
    ///
    ///    [ index of i_literal ] [ number ] in a compiled word
    ///
    pub fn i_literal(&mut self) {}

    /// Places the address (in string space) of the adjacent string on the stack
    ///
    ///    [ i_string ] [ index into string space ] in a compiled word
    ///
    pub fn i_string(&mut self) {}

    /// Loops through the adjacent definition, running their inner interpreters
    ///
    ///    [ index of i_definition ] [ sequence of compiled words ]
    ///
    pub fn i_definition(&mut self) {}
}

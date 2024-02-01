# tForth - a minimal Forth in Rust
## by Tim Barnes

tForth is a simple implementation of some basic Forth language capabilities. Where possible, I have followed the [Forth standard:](https://forth-standard.org). My intent was to simultaneously learn Forth and Rust. 

The program relies on a Rust binary, and a source file containing library functions (currently quite small).
The program should be installed anywhere, but the default location for the library file is '~/.tforth/corelib.fs'.

## Command line Options
tForth responds to the following command line options:
'Usage: tforth [OPTIONS]

Options:
      --debuglevel <VALUE>  [possible values: error, warning, info, debug]
      --library <VALUE>     
      --file <VALUE>        
      --nocore              
  -h, --help                Print help
  -V, --version             Print version'

  '--library' allows a library other than the core library to be installed.
  '--nocore' suppresses loading of the core library
  '--file' allows a user-defined file to be loaded, after (or without) the core file.

  

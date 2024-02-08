( Core word definitions )

: negate ( n -- -n ) if -1 else 0 then ;
: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a b ) over over ;
: ?dup dup 0= if dup else then ;
: > < if false else true then ;
: <> ( n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs (n -- n | -n ) dup 0 < if -1 * then ;
: dbg-debug 3 dbg ;
: dbg-info 2 dbg ;
: dbg-warning 1 dbg ;
: dbg-quiet 0 dbg ;
: debug show-stack step-on ;
: bl 32 ; ( puts the character code for a space on the stack )
: 1- ( n -- n-1 ) 1 - ;
: 1+ ( n -- n+1 ) 1 + ;
: endif then ; ( synonym for then, to allow if - else - endif conditionals )

: +! ( n addr -- ) dup @ rot + swap ! ;
: ? ( addr -- ) @ . ;

: run-regression clear s" src/regression.fs" loaded ;


( Application functions )
: fac ( n -- n! )   \ Calculates factorial of a non-negative integer. No checks for stack or calculation overflow.
    dup 
        if 
            1 swap _fac 
        else 
            drop 1 
        then ;

: _fac ( r n -- r )   \ Helper function that does most of the work.
    dup if 
        tuck * swap 1 - _fac 
    else 
        drop 
    then ;

." Library loaded."

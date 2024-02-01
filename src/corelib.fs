( Core word definitions )

: negate ( n -- -n ) if -1 else 0 then ;
: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a b ) over over ;
: ?dup dup 0= if dup else then ;
: > < negate ;
: <> (n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs (n -- n | -n ) dup 0 < if -1 * then ;
: false ( -- -1 ) -1 ;
: true ( -- 0 ) 0 ;
: dbg-debug 3 dbg ;
: dbg-info 2 dbg ;
: dbg-warning 1 dbg ;
: dbg-quiet 0 dbg ;
: bl 20 ; ( puts the character code for a space on the stack )
: cr 10 ; ( puts the character code for a carriage return on the stack )
: 1- ( n -- n-1 ) 1 - ;
: 1+ ( n -- n+1 ) 1 + ;
: endif then ; ( synonym for then, to allow if - else - endif conditionals )

( Test Functions: place desired result on the stack, 
  then push args to the test word, then the word, then test-single.
  If the desired result is equal to the top of the stack, the test passes. )
: test-single ( m n.. -- b ) = if ." Passed" else ." Failed"  then ;
: test-dual ( j k n.. -- b ) rot = rot rot = and if ." Passed" else ." Failed" then ;

: fac ( n -- n! ) 
    dup 
        if 
            1 swap _fac 
        else 
            1 
        then ;

: _fac ( r n -- r ) 
    dup if 
        tuck * swap 1 - _fac 
    else 
        drop 
    then ;

." Library loaded."

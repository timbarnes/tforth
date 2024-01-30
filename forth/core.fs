( Core word definitions )
: negate ( n -- -n ) if -1 else 0 then ;
: nip ( a b -- b ) swap drop ;
: tuck ( a b -- b a b ) swap over ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a ) over over ;
: ?dup dup 0= if dup else then ;
: <> (n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;
: abs (n -- n | -n ) dup 0 < if -1 * then ;
: false ( -- -1 ) 0 ;
: true ( -- 0 ) -1 ;
: debug 3 dbg ;
: info 2 dbg ;
: warning 1 dbg ;
: quiet 0 dbg ;
: bl 20 ; ( puts the character code for a space on the stack )


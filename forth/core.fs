( Core word definitions )
: negate ( n -- -n ) if -1 else 0 then ;
: nip ( a b -- b ) swap drop ;
: pop ( a -- ) drop ;
: 2dup ( a b -- a b a ) over over ;
: <> (n -- n ) = 0= ;
: min ( m n -- m | n ) 2dup < if drop else nip then ;
: max ( m n -- m | n ) 2dup > if drop else nip then ;

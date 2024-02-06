( Washing machine controller )

: run-machine
	check-door
	if lock-door run-cycles unlock-door
	else ." Please close door"
	then ;

: check-door
	door-closed @
	if ." Door is closed." true
	else false
	then ;

: lock-door
	true door-locked ! ." Door has been locked." ;

: unlock-door
	false door-locked ! ." Door has been unlocked." ;

: check-lock door-locked @ ;

: run-cycles wash rinse spin beep ;

: wash ." Washing..." ;
: rinse ."  Rinsing..." ;
: spin check-lock if ."   Spinning..." else ." Door is not locked" then ;
: beep ."     --BEEP!--" ;

variable door-closed
	false door-closed ! ( Door is initially open )
variable door-locked
	false door-locked ! ( Door is initially unlocked)

: close-door true door-closed ! ." Door has been closed." ;


	

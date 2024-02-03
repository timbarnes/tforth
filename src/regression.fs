( Regression tests )
( Run with standard library loaded )
( test-none is for words leaving no words on the stack )
( test-single is for words leaving a single value on the stack )
( test-dual is for words leaving two values on the stack )
( failing tests leave the test number on the stack )
( test-results checks the stack to see if there were failures )

variable test-num 0 test-num !

: loop-test do i .. ." , " loop ;
: nested-loop-test 
    do 
        6 4 do 
            i .. ."  inner " 
            loop i .. ."   outer " 
    loop ;

."         Clear has to be the first test"
1 2 3 4 5 clear test-none
7 0 loop-test test-none
-4 -8 nested-loop-test test-none
-22 33 loop-test test-none
33 -22 loop-test test-none
0 0 loop-test test-none

."         Printing"
23 . test-none
44 .. flush test-none
.s test-none
45 emit test-none

."         Debugger"
0 dbg test-none ." quiet mode (errors only)"
1 dbg test-none ." warnings and errors"
2 dbg test-none ." info, warnings and errors"
3 dbg test-none ." debug, info, warnings and errors"
4 dbg test-none ." invalid value 4"
-4 dbg test-none ." invalid value -4"
dbg-warning test-none
debuglevel? test-none 

."         Arithmetic"
5 1 4 + test-single
-10 5 15 - test-single
-20 2 -10 * test-single
4 12 3 / test-single

."         Logic"
-1 true test-single
0 false test-single

."         Comparisons"
false 1 3 > test-single ." >"
true 3 1 > test-single
false 5 2 < test-single
true 2 5 < test-single
false -5 0= test-single
true 0 0= test-single
true -22 0< test-single
false 0 0< test-single
false 55 0< test-single

."         Bitwise"
3 1 2 or test-single ." or"
2 0 2 or test-single
0 0 0 or test-single
3 -1 3 and test-single ." and"
0 0 0 and test-single
0 0 3 and test-single
45 -1 45 and test-single

."        Stack operations"
100 drop test-none
5 5 6 drop test-single
5 6 5 nip test-single
1 2 3 4 5 6 stack-depth test-single drop drop drop drop drop
17 17 17 dup test-dual
4 7 7 4 swap test-dual
5 12 5 12 over drop test-dual
9 4 6 9 4 rot drop test-dual

."        Variables"
5 variable x 5 x ! x @ test-single
42 variable y 40 y ! 2 y +! y @ test-single
variable z 42 z ! z ? test-none

."        Application tests"
1 0 fac test-single
1 1 fac test-single
6 3 fac test-single
479001600 12 fac test-single

test-results
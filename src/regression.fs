( Regression tests )
( Run with standard library loaded )
( test-none is for words leaving no words on the stack )
( test-single is for words leaving a single value on the stack )
( test-dual is for words leaving two values on the stack )
( failing tests leave the test number on the stack )
( test-results checks the stack to see if there were failures )

variable test-num 0 test-num !

( Test Functions: place desired result on the stack, 
  then push args to the test word, then the word, then test-single.
  If the desired result is equal to the top of the stack, the test passes.
  Relies on a variable test-num that indicates the number of the test. )

variable test-num 0 test-num !
: test-none ( .. -- ) stack-depth 1 test-num +! 
    0= if test-num ? ."  Passed" else ."    Failed" test-num @ then ;

: test-single ( m n.. -- b ) 1 test-num +! 
    = if test-num ? ."  Passed" else ."    Failed"  test-num @ then  ;

: test-dual ( j k n.. -- b ) 1 test-num +!
    rot = 
    rot rot = 
    and if test-num ? ."  Passed" else ."    Failed" test-num @ then ;

: test-results stack-depth 0= if ." All tests passed!" else ." The following tests failed: " .s clear then ;

: loop-test do i loop ;
: nested-loop-test 
    do 
        6 4 do 
            i . j .
            loop i . 
        i 
    loop ;

: loop+test do i dup dup . +loop ;

: leave-test 
    do 
        i dup 
        if 
            ." inner" 
        else 
            ." leaving" leave 
        then 
            i . 
    loop ;

."         Clear has to be the first test"
1 2 3 4 5 clear test-none

."         Debugger"
1 1 1 dbg test-single ." warnings and errors"
1 1 2 dbg test-single ." info, warnings and errors"
1 1 3 dbg test-single ." debug, info, warnings and errors"
1 1 0 dbg test-single ." quiet mode (errors only)"
1 1 4 dbg test-single ." invalid value 4"
1 1 -4 dbg test-single ." invalid value -4"
1 1 dbg-warning test-single
1 1 debuglevel? test-single 

."         Printing"
1 1 23 . test-single
1 1 44 . flush test-single
1 1 .s test-single
1 1 45 emit test-single

."                Loop tests"
0 21 7 0 loop-test + + + + + test-dual
1 6 4 1 nested-loop-test * test-dual
1 -2 1 loop-test test-single
3 15 7 3 loop-test + + test-dual
0 0 0 loop-test test-single
1 64 10 1 loop+test * * test-dual
-1 0 5 -1 leave-test test-dual


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
 1 1 100 drop test-single
5 5 6 drop test-single
5 6 5 nip test-single
1 1 2 3 4 5 6 stack-depth drop drop drop drop drop drop test-single
17 17 17 dup test-dual
4 7 7 4 swap test-dual
5 12 5 12 over drop test-dual
9 4 6 9 4 rot drop test-dual

."        Variables"
5 variable x 5 x ! x @ test-single
42 variable y 40 y ! 2 y +! y @ test-single
42 variable z 42 z ! z ? z @ test-single

."        Constants"
12 12 constant months months test-single \ a constant with the value 12

."        Application tests"
1 0 fac test-single
1 1 fac test-single
6 3 fac test-single
479001600 12 fac test-single 

test-results  \ Checks to see if all tests passed. Errors, if any, are left on the stack.

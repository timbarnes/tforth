( Regression tests )
( Run with standard library loaded )
( test-none is for words leaving no words on the stack )
( test-single is for words leaving a single value on the stack )
( test-dual is for words leaving two values on the stack )

variable test-num 0 test-num !

100 drop test-none
23 . test-none

1 2 3 4 5 6 stack-depth test-single clear
5 1 4 + test-single
-10 5 15 - test-single
-20 2 -10 * test-single
4 12 3 / test-single

17 17 17 dup test-dual
-1 -5 -5 0= test-dual
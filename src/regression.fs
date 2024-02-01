( Regression tests )
( Run with standard library loaded )
( test-single is for words leaving a single value on the stack )
( test-dual is for words leaving two values on the stack )

variable test-num 0 test-num !

5 1 4 + test-single
-10 5 15 - test-single
-20 2 -10 * test-single
4 12 3 / test-single
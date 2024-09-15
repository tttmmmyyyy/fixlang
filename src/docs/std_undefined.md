An undefined value.

Since `undefined()` has generic type `a`, you can put it anywhere and it will be type-checked.
This is useful as a placeholder value that you haven't implemented yet.

Calling this value aborts the execution of the program (calls `abort` in libc).
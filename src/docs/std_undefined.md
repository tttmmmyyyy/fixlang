Generates an undefined value.

Calling this function prints `msg` to the stderr, flush stderr, and aborts the program (calls `abort` in libc).
Since `undefined(msg)` has generic type `a`, you can put it anywhere and it will be type-checked.
This is useful when you want to write a placeholder that will be implemented later.
Generates an undefined value.

Calling this function prints `msg` to the stderr, flush stderr, and aborts the program (calls `abort` in libc).
Since `undefined(msg)` has generic type `a`, you can put it anywhere and it will be type-checked.

This is useful when you want to write a placeholder that will be implemented later:

```
truth : I64;
truth = undefined("I will implement the truth later.");
```

Another use case is aborting the program when a certain branch of the code should not be reached:

```
if condition {
    // Do something.
} else {
    undefined("This branch should not be reached.");
}
```
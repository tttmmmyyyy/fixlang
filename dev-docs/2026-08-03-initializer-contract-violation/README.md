# A program that violates the global-initializer rule

`Array::mutate_elements` hands its callback a raw pointer into the array's element buffer, and the
callback may run any IO. That is enough to move a Fix object out of the program's sight and back in
somewhere else, which is how a global value's initializer can come to hold an object the running
program still holds.

`main.fix` here does exactly that, and it is what the documentation of `Array::mutate_elements`
points at. Build it with

    gcc -c -o steal.o steal.c && fix build && ./a.out

## What it does

1. `main` builds `victim`, an array the compiler proves local: the reference counting on it is
   compiled to a plain increment and decrement, with no read of the object's state byte.
2. `main` puts `victim` in another array and calls `mutate_elements` on it. The callback hands the
   element buffer to C, which keeps a copy of the three words that make up `victim`.
3. `main` reads the global `planted`, which runs its initializer for the first time. The initializer
   calls `mutate_elements` on an array of its own, and its callback asks C to write the kept words
   back — so the array the initializer returns holds `victim`.
4. Finishing an initializer marks its whole result graph global, which exempts those objects from
   reference counting. `victim` is now such an object.
5. `victim` is still live in `main`, and the reference counting on it was compiled on the proof that
   it is local. It decrements a count that is no longer maintained.

Note that step 3 needs no escape hatch: `mutate_elements` is not an IO action. It takes an IO action
and runs it, so a global's initializer can call it in ordinary pure code.

## What goes wrong

The program prints `11` and exits 0 as it stands, because the count `victim` loses does not reach
zero and nothing frees it. The violation is real all the same: a compiler built with the
development-mode state check unconditional aborts on it with

    A reference-counting operation inferred local reached a non-local object.

A program that arranged for the count to reach zero would free an object it still uses.

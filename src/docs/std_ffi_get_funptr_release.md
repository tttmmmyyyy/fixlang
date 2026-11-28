Returns a pointer to the function of type `void (*)(void*)` which releases a boxed value of type `a`.
This function is used to release a pointer obtained by `boxed_to_retained_ptr`.

This function takes a value of type `Lazy a`, which only names the type `a`.
So you can get the release function for a boxed type `T` even when you don't have a value of type `T` -- you can just use `|_| undefined("") : T`:

```fix
# module DocTest;
#
type VoidType = box struct {};
// No constructor for `VoidType` is provided.

main: IO ();
main = (
    let release = (|_| undefined("") : VoidType).get_funptr_release; // Release function of `VoidType`.
    pure()
);
```

# Parameters

* `lazy_value` - The lazy boxed value to indicate the type of the boxed value to be released.
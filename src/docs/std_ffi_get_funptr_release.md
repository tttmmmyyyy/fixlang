Returns a pointer to the function of type `void (*)(void*)` which releases a boxed value of type `a`.
This function is used to release a pointer obtained by `boxed_to_retained_ptr`.

Note that this function is requires a value of type `Lazy a`, not of `a`.
So you can get release function for a boxed type `T` even when you don't have a value of type `T` -- you can just use `|_| undefined("") : T`:

```
module Main;

type VoidType = box struct {};
// No constructor for `VoidType` is provided.

main: IO ();
main = (
    let release = (|_| undefined("") : VoidType).get_funptr_release; // Release function of `VoidType`.
    pure()
);
```

In case the type is not a specific `T`, but a generic parameter `a` that appears in the type signature of a function you are implementing, you cannot use the above technique, because writing `|_| undefined("") : a` is not allowed in Fix's syntax. Even in such a case, if you have some value related to `a`, you can make a `Lazy a` value in many cases. For example:
- If you have a function `f : b -> a`, then you can use `|_| f(undefined(""))` of type `Lazy a`. 
- If you have a function `f : a -> b`, then you can use `|_| let x = undefined(""); let _ = f(x); x` of type `Lazy a`.

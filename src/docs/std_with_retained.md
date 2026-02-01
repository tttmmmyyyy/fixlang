`x.with_retained(f)` runs `f` with retained `x`. 
It is guaranteed that `x` is keep alive until `with_retained` is finished, even after `f` has finished using `x` in it. 

A typical use case of this function is the implementation of `Std::FFI::borrow_boxed`.

# Parameters

* `f` - The function to run with the retained value.
* `x` - The value to retain.
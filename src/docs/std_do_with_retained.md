`x.do_with_retained(f)` runs `f` with retained `x`. 
It is guaranteed that `x` is keep alive until `do_with_retained` is finished, even after `f` has finished using `x` in it. 

A typical use case of this function is the implementation of `Std::Array::borrow_ptr`.
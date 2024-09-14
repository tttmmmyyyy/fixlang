`loop` enables you to make a loop. `LoopResult` is a union type defined as follows: 

```
type LoopResult s r = unbox union { continue : s, break : r };
```

`loop` takes two arguments: the initial state of the loop `s0` and the loop body function `body`. 
It first calls `body` on `s0`. 
If `body` returns `break(r)`, then the loop ends and returns `r` as the result. 
If `body` returns `continue(s)`, then the loop calls again `body` on `s`.

Example:
```
module Main;
    
main : IO ();
main = (
    let sum = loop((0, 0), |(i, sum)|
        if i == 100 { break $ sum };
        continue $ (i + 1, sum + i)
    );
    println $ sum.to_string
); // evaluates to 0 + 1 + ... + 99 
```
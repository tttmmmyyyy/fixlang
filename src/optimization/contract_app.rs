/*
Contract application optimization.

This optimization tries to reduce cost of "create lambda and apply" expressions.

1. Moves application of variable into inner expression.

For example, if `v` is a variable,

```
(if c {{expr0}} else {{expr1}})(v)
```

is transformed into

```
if c {{expr0}(v)} else {{expr1}(v)}
```

2. Replaces application of lambda expression to a variable expression with substitution.

For example, if `v` is a variable,
```
(|x| {expr})(v)
```

is transformed into

```
{expr}[v/x]
```
*/

use crate::{InstantiatedSymbol, Program};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.instantiated_symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(_sym: &mut InstantiatedSymbol) {}

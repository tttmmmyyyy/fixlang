/*
# Index Syntax Optimization

## Overview

This optimization detects patterns of AST nodes created using index syntax and transforms them into more efficient forms.

AST nodes created using index syntax have the following form:

```
use(|f| obj.act0(act1(...actN(f)...)))
```

- `use` represents one of `Std::Indexable::iget`, `Std::Indexable::iset(v)`, or `Std::Indexable::imod(v)`.
- `act{I}` represents `Std::Array::act(i)` or a struct field accessor `act_{field}`.

When this pattern is found, it is transformed into the following expression:

When `use` is `Std::Indexable::iget`:
```
obj.get0.get1....getN
```
Here, `get{I}` represents `Std::Array::@(i)` or a struct field accessor `get_{field}`.

When `use` is `Std::Indexable::iset(v)`:
```
obj.mod0(mod1(...modN-1(setN(v))...))
```

When `use` is `Std::Indexable::imod(v)`:
```
obj.mod0(mod1(...modN(v)...))
```

*/

use crate::ast::program::Program;

pub fn run(prg: &mut Program) {}

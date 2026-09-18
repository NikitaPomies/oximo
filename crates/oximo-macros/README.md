# oximo-macros

Internal procedural macros backing oximo's modeling surface:
`variable!`, `constraint!`, `soc_constraint!`, `objective!`, `sum!`, `set!`,
and `param!`.

This crate is an implementation detail, do not depend on it directly.
The macros are re-exported through `oximo-core` and `oximo::prelude`, which is the
supported entry point:

```rust,ignore
use oximo::prelude::*;
```

The macros expand to the typed builder API in `oximo-core` (`Model`, `Set`,
`Expr`, `sum_over`, ...). See the `oximo` crate docs for the macro grammar and examples.

Sums inside `constraint!`, `soc_constraint!`, and `objective!` inherit the model's
expression context and return `0` for empty domains or filters. For standalone
sums, use `sum!(model, body for i in domain)` to supply that context explicitly.
Nested sums inherit it, and selected terms are checked for model ownership.
Standalone unanchored sums still require at least one term.

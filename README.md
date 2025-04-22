# `stonkscheme`

A toy scheme interpreter that aims to implement a subset of something like PowerLanguage or EasyLanguage. I think I was mostly amused by
the name, and this isn't meant to be useful.

```scheme
> (car (1 2))
Integer (1)
> (cdr (1 2 3))
Combination (Integer (2) ,[Integer (3)])
```

Current status: s-expression parser with a minimal evaluator and a REPL. The special forms required to run `supersmoother.scm` are not
implemented yet. Current special forms:

* `set`
* `get`
* `car`
* `cdr`
* `cons`
* `if`

+ `+`
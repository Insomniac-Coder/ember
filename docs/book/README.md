# The Ember guide

Ember reads like Python and runs like C. This guide is for someone who has written a little Python
(or another language) and wants to write Ember: it starts at a first program and ends at modules,
tests and the tools, one idea at a time. The specification (`docs/spec-source/`) says exactly what
every construct means; the guide shows how to use them.

1. [A first program](01-first-program.md) — `println`, running a file, `main`, checking without
   running.
2. [Values and control flow](02-values-and-control.md) — names, numbers, booleans, `if`, loops.
3. [Functions](03-functions.md) — parameters, defaults, named arguments, several results, lambdas.
4. [Collections](04-collections.md) — `Array`, `Map`, `Set`, slices and comprehensions.
5. [Text](05-text.md) — `str` and `String`, f-strings, splitting, searching, parsing.
6. [Structs, enums and `match`](06-structs-enums-match.md) — your own types, and `Option`.
7. [Errors](07-errors.md) — `Result`, `?`, and panics.
8. [Ownership](08-ownership.md) — moves, borrowing, `mut` and `owned` parameters, `clone`.
9. [Classes](09-classes.md) — objects shared by handles, inheritance, `Weak`.
10. [Modules and packages](10-modules-and-packages.md) — `import`, the standard library, `ember.toml`.
11. [Tests and tools](11-tests-and-tools.md) — `@test`, `ember test`, `fmt`, `lint`, `explain`.
12. [Coming from Python](12-coming-from-python.md) — what carries over, what is spelled
    differently, what is new; one program written in both.

## The samples

Every sample marked as Ember is a whole program, and `ember test --doc` builds and runs each one:
it must exit with status 0, and when an `output` block follows it, print exactly that. A sample
marked `ember,fragment` is part of a program (it names things it does not declare) and must parse.
So the guide cannot say something the compiler does not do:

```text
ember test --doc docs/book/
```

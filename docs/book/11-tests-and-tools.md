# 11. Tests and tools

## Tests

A function marked `@test` is a test: no parameters, no result, and it passes unless it panics.
`assert(condition)` and `assert(condition, message)` panic when the condition is false.

```ember,fragment
fn median(values: Array[int]) -> int:
    ordered = sorted(values)
    return ordered[ordered.len() // 2]

@test
fn median_of_an_odd_count():
    assert(median([5, 1, 3]) == 3)

@test
@should_panic
fn median_of_nothing_panics():
    empty: Array[int] = []
    median(empty)
```

`ember test` builds the package's tests and runs each in a process of its own, so one crashing
says nothing of another, and prints each one's result; `ember test median` runs those whose name
contains `median`. `@should_panic` passes only if the test panics. In the `debug` profile each test
also reports the objects it leaked in a cycle (chapter 9).

`ember test --doc` runs the documentation's examples instead — the error pages' and this guide's.

## The other tools

| Command | What it does |
|---|---|
| `ember check` | checks a program without building it |
| `ember run --interp` | runs it on the evaluator, without a C compiler |
| `ember fmt file.em` | prints the file in the one layout (`--write` rewrites it, `--check` says whether it is) |
| `ember lint` | the checks of `ember check`, and lints: an unneeded clone, a large copy, a lock held across a blocking call, allocation in a hot loop |
| `ember doc file.em` | each function's contract: how it takes each parameter, what its result borrows, its effects |
| `ember explain E3040` | the page about an error code; `ember explain --borrow file.em:12` why each borrow alive at line 12 is still alive |
| `ember inspect name --cost` | what a function costs, line by line: allocations, checks, copies |
| `ember why --alloc name file.em` | the shortest call chain from a function to an allocation (also `--io`, `--block`, …) |
| `ember build --profile release` | an optimised program; `debug` (the default) keeps every check and adds leak reports |

Every command takes `--json` for tools, and exits with a status other than 0 when it reports an
error.

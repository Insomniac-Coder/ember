# 1. A first program

Save this as `hello.em`:

```ember
println("hello, world")
```

```output
hello, world
```

and run it:

```text
ember run hello.em
```

`ember run` checks the program, compiles it to C, builds it with the C compiler it finds (MSVC,
Clang or GCC) and runs it. A single file needs no project around it.

## Scripts and `main`

Statements written at the top of the file run in order, as in a Python script. A larger program
usually puts them in a function named `main`, which runs when the program starts:

```ember
fn main():
    name = "Ada"
    println("hello,", name)
```

```output
hello, Ada
```

`println` prints its arguments separated by spaces and ends the line; `print` does not end it.
Blocks are indented with spaces after a line ending in `:`, and a `#` starts a comment.

## Checking without running

```text
ember check hello.em
```

checks types, borrowing and effects and reports every error, without building anything. An error
names the rule it comes from and, most often, the change that fixes it:

```text
error[E2020]: expected `i64`, found `str`
```

`ember explain E2020` prints the page about any error code.

## A project

```text
ember new hello
cd hello
ember run
```

`ember new` makes a folder with `ember.toml` (the package's name and settings) and `src/main.em`.
Inside it `ember run`, `ember build`, `ember check` and `ember test` need no file name.

## Running without a C compiler

```text
ember run --interp hello.em
```

runs the program on Ember's own evaluator — the one that computes values while compiling — instead
of building it. It needs no C compiler and prints what the built program prints; a program that
calls C or starts a thread is refused there (`E6010`).

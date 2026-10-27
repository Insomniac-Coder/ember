#$ test: compile-fail
#$ rules: R12
#$ help: [C] above fn each_name(owns f: fn(owns String) changes C) changes C:
## Verdicts 21 and 22, R12 (rules.md lines 537-549): a parameter of type `fn(owns String)` takes only
## functions that change nothing; a function that runs a function it is given takes a change list.
## AN-56 part 6's message, word for word.

fn each_name(owns f: fn(owns String)):
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))    #$ error[E2113]: this function changes `names`, a variable it captured (`fn(owns String) changes names`), and `each_name` takes `fn(owns String)`, functions that change nothing; let it take one that changes things, and say that a call makes those changes: `[C]` above `fn each_name(owns f: fn(owns String) changes C) changes C:`
    println(names.len())

#$ test: compile-fail
#$ rules: FN-8
#$ help: end a script early with `process.exit(code)`
# `[FN-8]` — `return` and `yield` at file scope are `E0100`, as in Python.

x = 1
if x > 0:
    return    #$ error[E0100]: `return` outside a function
yield x    #$ error[E0100]: `yield` outside a function

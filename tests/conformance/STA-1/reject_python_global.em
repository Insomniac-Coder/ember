#$ test: compile-fail
#$ rules: STA-1
# `[STA-1]` — Python's `global` is `E0100`: a function changes a module variable by its name, with
# no word.

static mut count: int = 0

fn bump():
    global count    #$ error[E0100]: `global` is Python's; in Ember a function changes a module variable by its name, with no word: remove line 9
    count += 1

fn main():
    bump()

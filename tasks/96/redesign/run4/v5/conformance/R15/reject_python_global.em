#$ test: compile-fail
#$ rules: R15, R5
## R15 and R5 (verdict 86): a function changes a module variable by its name, with no word; Python's
## `global` is refused with the line to remove (AN-76; section 3's E0100 message).

static mut FRAME: int = 0

fn tick():
    global FRAME    #$ error[E0100]: `global` is Python's; in Ember a function changes a module variable by its name, with no word: remove line 9
    FRAME += 1

fn main():
    tick()

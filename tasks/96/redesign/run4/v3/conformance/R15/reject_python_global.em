#$ test: compile-fail
#$ rules: R15, R5
## R15 and R5: a function that changes a module variable says so on its first line; Python's `global`
## is refused with that line (AN-76).

static mut FRAME: int = 0

fn tick():
    global FRAME    #$ error[E0100]: `global` is Python's; in Ember a function that changes a module variable says so on its first line: remove line 9 and write `fn tick() changes FRAME:` instead of line 8
    FRAME += 1

fn main() changes FRAME:
    tick()

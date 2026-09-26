#$ test: compile-fail
#$ rules: STA-1
#$ error[E7002]: static type `cstr` is not Sync

static greeting: cstr = c"hello"

fn main():
    pass

#$ test: compile-fail
#$ rules: STA-1
#$ error[E7002]: static type `CStrView` is not Sync

static greeting: CStrView = c"hello"

fn main():
    pass

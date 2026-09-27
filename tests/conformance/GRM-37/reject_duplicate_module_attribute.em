#$ test: compile-fail
#$ rules: GRM-37, ATT-4
#$ profiles: debug
#$ error[E0104]: duplicate `#! module` directive

#! module overflow(wrap)
#! module overflow(panic)

fn main():
    pass

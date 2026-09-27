#$ test: compile-fail
#$ rules: GRM-37, ATT-4
#$ profiles: debug
#$ error[E0104]: `#! module deterministic` takes no arguments
#$ error[E0104]: `#! module reloadable` takes no arguments

#! module deterministic(extra)
#! module reloadable(extra)

fn main():
    pass

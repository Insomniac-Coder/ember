#$ test: compile-fail
#$ rules: GRM-37
#$ profiles: debug
#$ error[E0104]: `#! module` needs one valid attribute name and arguments

#! module overflow(wrap) trailing

fn main():
    pass

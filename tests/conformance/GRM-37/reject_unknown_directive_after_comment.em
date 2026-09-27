# A directive must not be silently treated as a comment.
#! unrecognized value
#$ test: compile-fail
#$ rules: GRM-37
#$ error[E0104]: `#! unrecognized` is not a directive

fn main():
    pass

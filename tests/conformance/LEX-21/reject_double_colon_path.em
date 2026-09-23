#$ test: compile-fail
#$ rules: LEX-21, GRM-24
#$ profiles: debug
#$ error[E0100]: `::` is not a path separator
#$ help: use '.' for paths
# There is no `::` token: `.` is the one path separator.

enum Shape:
    Dot
    Square(int)

fn main():
    s = Shape::Square(2)

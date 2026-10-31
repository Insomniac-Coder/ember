#$ test: compile-fail
#$ rules: STD-19, OWN-8
#$ profiles: debug
#$ error[E2040]: `copy_each` needs `File` to be `Clone`
## `[STD-19]` — `copy_each()` gives what `copy of` makes of each item, so the items must be `Clone`; a
## struct with a `drop` of its own derives no `Clone` unless it asks.

struct File:
    fd: int = 0

    fn drop():
        pass

fn main():
    files: Array[File] = [File(fd=1)]
    println(files.iter().copy_each().count())

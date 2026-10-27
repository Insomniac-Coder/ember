#$ test: compile-fail
#$ rules: BRW-17
## design rules: R9, R8
## R9, R8 (a), (b): a removal made through any owner of a shared list ends the views of its elements;
## two owners of one list are two names of one place.

fn main():
    names = [String("a"), String("b")]
    other = shares mut names
    first = names[0]
    other.remove(0)
    println(first)                #$ error[E3060]: `other.remove(0)` (line 10) may put another string in the place `first` names (line 9), since `other` and `names` own one list, and `first` is used on line 11; use `first` before line 10, or name the string again after it: write `first = names[0]` above line 11

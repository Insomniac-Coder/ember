#$ test: compile-fail
#$ rules: R9, R8
## R9, R8 (a), (b): a removal made through any owner of a shared list ends the views of its elements;
## two owners of one list are two names of one place. `main` names the list it changes through the link
## `other` (R5, verdict 64).

fn main() changes Array[String]:
    names = [String("a"), String("b")]
    other = shares mut names
    first = names[0]
    other.remove(0)
    println(first)                #$ error[E3060]: `other.remove(0)` (line 11) may put another string in the place `first` names (line 10), since `other` and `names` own one list, and `first` is used on line 12; use `first` before line 11, or name the string again after it: write `first = names[0]` above line 12

#$ test: compile-pass
#$ rules: R9, R8, R5
## R9, R8 (a): a view's life ends at its last use, so a removal through another owner after it is
## allowed. The removal goes through the link `other`, so `main` says `changes Array[String]` (R5,
## verdict 64).

fn main() changes Array[String]:
    names = [String("a"), String("b")]
    other = shares mut names
    first = names[0]
    println(first)
    other.remove(0)

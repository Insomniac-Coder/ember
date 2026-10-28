#$ test: compile-pass
#$ rules: WK-11
#$ warning[L3001]: `Person.best` and `Person.friends` can hold each other: a cycle of `shares` owners is never freed
#$ help: a link that should not keep its person alive is `weakref`: `best: weakref Person = None`
## design rules: R11
## R11: "The cycle warning (L3001) is given for a type whose `shares` fields can form a cycle of
## owners, and names the link to make `weakref`" (AN-64). A warning, not a refusal: a leak, not a
## memory error.

class Person:
    name: String
    friends: Array[shares Person] = []
    best: Option[shares Person] = None

fn main():
    pass

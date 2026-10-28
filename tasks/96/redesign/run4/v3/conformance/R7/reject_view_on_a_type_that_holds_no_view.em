#$ test: compile-fail
#$ rules: R7
#$ help: remove `@view`
## R7: `@view` is checked: on a type that holds no view it is refused (today's E2030, kept by the
## vocabulary's `@view` row; see the matrix, R7-2).

@view    #$ error[E2030]: `@view` on `Point`, which holds no view; remove `@view`
struct Point:
    x: int
    y: int

fn main():
    println(Point(x=1, y=2).x)

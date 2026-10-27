#$ test: compile-fail
#$ rules: R7
#$ help: remove `@view`
## R7: `@view` is checked: on a type that holds no view it is refused (E2030, today's code, in
## rules section 4; section 3's message word for word).

@view    #$ error[E2030]: `@view` on `Point`, which holds no view; remove `@view`
struct Point:
    x: int
    y: int

fn main():
    println(Point(x=1, y=2).x)

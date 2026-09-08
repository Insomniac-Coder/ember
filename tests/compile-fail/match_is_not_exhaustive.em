#$ test: compile-fail
#$ rules: ENM-2, CTL-5
#$ error[E2090]: `match` on `Color` does not cover every value
#$ error[E2090]: `Color.Green`, `Color.Blue` not covered
#$ error[E2090]: `match` on `Shape` does not cover every value
#$ error[E2090]: `Shape.Rect(_, _)` not covered

enum Color:
    Red
    Green
    Blue

enum Shape:
    Circle(f32)
    Rect(f32, f32)

fn partial(c: Color) -> i32:
    match c:
        Color.Red:
            return 1
    return 0

fn payload(s: Shape) -> i32:
    match s:
        Circle(r):
            return 1
    return 0

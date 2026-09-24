#$ test: run-pass
#$ rules: CTL-10, GRM-4
#$ stdout:
#$ -2 is negative 0 is zero 7 is positive
#$ 18 -1 1
#$ short a
#$ double bb
#$ circle 3.14 square 4.0 dot
# `[CTL-10]` — a name not in scope before an `if` with an `else`, or an
# exhaustive `match` statement, and declared by `x = e` in every arm that
# completes normally at one type, is declared after the statement. An arm
# that returns, breaks, continues or panics need not declare it; a nested
# branch hoists into its arm first.

enum Shape:
    Circle(float)
    Square(float)
    Dot

fn describe(n: int) -> String:
    if n < 0:
        kind = "negative"
    elif n == 0:
        kind = "zero"
    else:
        kind = "positive"
    return f"{n} is {kind}"

fn pick(n: int) -> int:
    if n > 5:
        v = n * 2
    elif n < 0:
        return -1
    else:
        v = 1
    return v

fn area_name(s: Shape) -> String:
    match s:
        Circle(r):
            name = "circle"
            area = 3.14 * r * r
        Square(w):
            name = "square"
            area = w * w
        Dot:
            return "dot"
    return f"{name} {area}"

fn main():
    println(describe(-2), describe(0), describe(7))
    println(pick(9), pick(-3), pick(2))
    names: Array[String] = ["a", "bb"]
    for w in names:
        if w.len() > 1:
            if w == "bb":
                tag = f"double {w}"
            else:
                tag = f"long {w}"
        else:
            tag = f"short {w}"
        println(tag)
    println(area_name(Shape.Circle(1.0)), area_name(Shape.Square(2.0)), area_name(Shape.Dot))

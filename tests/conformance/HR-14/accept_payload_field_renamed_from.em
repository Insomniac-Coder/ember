#$ test: run-pass
#$ rules: HR-14
#$ stdout: 12
#$ assert-c: contains("{ \"radius\", \"r\", ")
# A payload's fields migrate by the rows a type's take, so a payload field takes `@renamed_from`
# as a type's does (`variant_field := {attribute} [IDENT ":"] type`): the variant's schema names
# the old name, whose value a reload carries over. The program runs as any other.

enum Shape:
    Dot
    Circle(@renamed_from("r") radius: int)

fn size(s: Shape) -> int:
    match s:
        Shape.Dot:
            return 0
        Shape.Circle(radius):
            return radius

fn main():
    println(size(Shape.Circle(12)) + size(Shape.Dot))

#$ test: compile-pass
#$ rules: LEX-17a
#$ warning[W2015]: float literal loses precision at `f32`
# A literal that receives `f32` and has more significant digits than `f32`
# keeps (nine) is W2015, whether the type came from an annotation or from the
# other operand. `0.1` at `f32` is not warned.

fn main():
    precise: f32 = 0.1234567891234
    plain: f32 = 0.1
    println(precise + plain)

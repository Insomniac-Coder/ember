#$ test: run-pass
#$ rules: TYP-8, TYP-30
#$ profiles: debug, release, shipping
#$ stdout: 127 -128 -128 81 127 -128 1

@overflow(saturate)
fn powers(a: i8, b: i8) -> void:
    println(2i8 ** 7, (-2i8) ** 7, (-3i8) ** 5,
        (-3i8) ** 4, (-4i8) ** 4, a ** 1, b ** 0)

fn main():
    powers(i8.MIN, 3i8)

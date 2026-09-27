#$ test: run-pass
#$ rules: TYP-8, TYP-28
#$ profiles: debug, release, shipping
#$ stdout: 127 -128 127 -128 127 0 127
#$ stdout: 255 0 255 127 -128 -128

@overflow(saturate)
fn signed() -> void:
    hi: i8 = 120
    lo: i8 = -120
    println(hi + 20i8, lo - 20i8, 20i8 * 20i8, -20i8 * 20i8,
        i8.MIN // -1i8, i8.MIN % -1i8, -i8.MIN)

@overflow(saturate)
fn unsigned() -> void:
    hi: u8 = 250
    zero: u8 = 0
    hi += 10u8
    zero -= 1u8
    product: u8 = 30u8 * 10u8
    signed_value: i8 = 120
    signed_value += 20i8
    negative_product: i8 = -50
    negative_product *= 3i8
    negative_subtraction: i8 = -120
    negative_subtraction -= 20i8
    println(hi, zero, product, signed_value, negative_product, negative_subtraction)

fn main():
    signed()
    unsigned()

#$ test: run-pass
#$ rules: TYP-6
#$ stdout: 44
#$ -1
#$ 255
#$ 3
#$ 0
#$ 127
#$ 16777216
#$ 1
#$ 0
#$ 1
#$ 7
#$ true
#$ 5
## `[TYP-6]` - integer to integer keeps the low bits; float to integer rounds toward zero, saturates, NaN is 0;
## integer to float rounds to nearest even; bool to integer; a unit-only enum to an integer.

enum Color:
    Red
    Green = 7

fn main():
    big: i64 = 300
    println(big as u8)                    # low bits: 300 = 0x12C
    small: u8 = 255
    println(small as i8)
    neg: i32 = -1
    println(neg as u8)
    println(3.99 as int)                  # toward zero
    nan = 0.0 / 0.0
    println(nan as int)
    huge = 1e30
    println(huge as i8)                   # saturates at the target's bound
    odd: i32 = 16777217
    println(odd as f32 as int)            # ties to even: 16777217 is half way between two f32s
    println(true as int)
    println(false as int)
    println(Color.Red as u8 + 1)
    println(Color.Green as u8)
    if char.from_u32(65) exists as ch:
        println(ch == 'A')
    ok: Result[i8, RangeError] = i8.try_from(5 as int)
    match ok:
        Ok(v):
            println(v)
        Err(_):
            println(0)

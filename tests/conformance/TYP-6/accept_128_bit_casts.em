#$ test: run-pass
#$ rules: TYP-6, TYP-1
#$ profiles: debug, release
#$ stdout:
#$ 99999996802856924650656260769173209088
#$ 1.7014118e+38 -1.7014118346046923e+38 true inf
#$ -1 170141183460469231731687303715884105727 -170141183460469231731687303715884105728 340282366920938463463374607431768211455 0 0
#$ 200 340282366920938463463374607431768211449 170141183460469231731687303715884105728 0 0 -1
#$ 1000000000000000019884624838656 -9007199254740993 9007199254740992.0 9007199254740994.0
# D-272, `[TYP-6]` — casts to and from `i128` and `u128`. A float converts
# toward zero and saturates at the type's bounds, NaN to 0; an integer to a
# float rounds to nearest, ties to even (2^53 + 1 is a tie between 2^53 and
# 2^53 + 2 and goes to 2^53); a narrower integer is sign- or zero-extended;
# a narrower target keeps the low bits.

fn main():
    hi: i128 = 170141183460469231731687303715884105727
    lo: i128 = -hi - 1
    top: u128 = 340282366920938463463374607431768211455
    println((1.0e38 as f32) as u128)
    one: u128 = 1
    println(hi as f32, lo as f64, (top >> 24) as f32 == (one << 104) as f32, top as f32)
    nan = 0.0 / 0.0
    println(-1.5 as i128, 3.9e40 as i128, -3.9e40 as i128, 3.9e40 as u128, -1.0 as u128, nan as i128)
    b: u8 = 200
    println(b as i128, (-7i64) as u128, (hi as u128) + 1, lo as u64, (one << 70) as u32, top as i128)
    tie: i128 = 9007199254740993
    println(1.0e30 as i128, -tie, (tie as f64), ((tie + 1) as f64))

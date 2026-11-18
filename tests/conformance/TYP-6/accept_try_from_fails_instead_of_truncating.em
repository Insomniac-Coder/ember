#$ test: run-pass
#$ rules: TYP-6
#$ stdout: 200
#$ none
#$ none
#$ -5
#$ none
#$ 7
## `[TYP-6]` - `T.try_from(x) -> Result[T, RangeError]` is the checked conversion: it fails where `as` would truncate.

fn show8(r: Result[u8, RangeError]):
    match r:
        Ok(v):
            println(v)
        Err(_):
            println("none")

fn showi(r: Result[i8, RangeError]):
    match r:
        Ok(v):
            println(v)
        Err(_):
            println("none")

fn main():
    big: int = 200
    show8(u8.try_from(big))
    show8(u8.try_from(big + 100))              # 300 does not fit
    neg: int = -1
    show8(u8.try_from(neg))                    # a negative number is no `u8`
    showi(i8.try_from(neg * 5))
    wide: u8 = 200
    showi(i8.try_from(wide))                   # an `u8` that passes `i8`'s top
    seven: u64 = 7
    match i32.try_from(seven):
        Ok(v):
            println(v)
        Err(_):
            println("none")

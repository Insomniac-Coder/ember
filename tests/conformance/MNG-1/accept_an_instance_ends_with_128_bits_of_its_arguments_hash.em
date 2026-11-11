#$ test: run-pass
#$ rules: MNG-1
#$ stdout: 7
#$ assert-c: contains("int32_t em_2idG2c2a47066b4fc729d207037bb273fa35(int32_t")
# `[MNG-1]` — a generic instance's symbol is `G` and the 32 hex digits of the BLAKE3 hash of the
# canonical spelling of its arguments: 128 bits, so the digits end at the bracket and not 16
# digits sooner.

[T]
fn id(x: T) -> T:
    return x

fn main():
    println(id(7))

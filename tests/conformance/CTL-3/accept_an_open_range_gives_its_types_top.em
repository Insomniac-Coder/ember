#$ test: run-pass
#$ rules: CTL-3, STD-19
#$ profiles: debug, release
#$ stdout:
#$ 250 251 252 253 254 255 | 250 251 252 253 254 255 |
#$ 0 Some(253) Some(254) Some(255)
#$ 0 Some(18446744073709551615) Some(4000000000000000000) Some(-117)
# `[CTL-3]` (D-526) — `a..` gives its type's maximum: the `for` over it and
# its iterator both step to the next value when it is asked for, not when the
# last one is given. Its iterator leaves values out at once (`skip_front`,
# `nth`), the maximum included.

fn main():
    for i in 250u8..:
        print(i, end=" ")
        if i == 255:
            break
    print("| ")
    for i in (250u8..).iter().take(6):
        print(i, end=" ")
    println("|")
    it = (250u8..).iter()
    println(it.skip_front(3), it.next(), it.next(), it.next())
    big = (5u64..).iter()
    print(big.skip_front((u64.MAX - 5) as u128), big.next(), "")
    print((0u64..).iter().nth(4000000000000000000), "")
    small = (-120i8..).iter()
    println(small.nth(3))

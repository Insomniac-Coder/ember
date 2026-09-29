#$ test: run-pass
#$ rules: OPT-2, CTL-3
#$ profiles: debug, release, shipping
#$ stdout:
#$ 25
#$ 9
# A range's limit is read once, when the loop starts: the loop changing the
# variable it came from does not move it. Versioning (`[OPT-2]`) copies the
# loop, `data[i]` having no bound proved, and each copy holds the limit in
# the same hidden local; that local is read in place of `m` only where `m`
# still holds the value copied, which the `m -= 1` in the loop ends.

fn total(data: Array[int], n: int) -> int:
    m = n
    sum = 0
    for round in 0..2:
        for i in 0..m:
            sum += data[i]
            if i == 0:
                m -= 1
    return sum

fn main():
    data: Array[int] = [1, 2, 3, 4, 5]
    println(total(data, 5))
    println(total(data, 3))

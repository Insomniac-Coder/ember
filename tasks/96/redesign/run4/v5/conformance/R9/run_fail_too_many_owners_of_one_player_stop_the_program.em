#$ test: run-fail
#$ rules: RC-1
#$ count_bits: 8
#$ panics: too many owners of a `Player` (255)
## design rules: R9
## R9 (verdicts 59, 96): "A count is checked before it rolls over: before each +1 of an owner count ... it
## is compared with its maximum, and at the maximum the program stops". This program is compiled with an
## 8-bit count in each object (`#$ count_bits: 8`), stored and compared as the real 32-bit one is, so the
## maximum is 255: `p` is one owner and the loop's first 254 rounds add 254 more, which brings the count
## to 255; round 255's `shares p` stops the program, where a count that rolled over would let one owner's
## end free the player the others still use (safety validation (b), hole 5). Rules section 2, the stop
## texts and the paragraph under them. Real programs keep the 32-bit count:
## `run_real_programs_keep_32_bit_counts.em`.

class Player:
    hp: int = 100

fn main():
    p = Player()
    owners: Array[shares Player] = []
    for _ in 0..255:
        owners.push(shares p)     # round 255 stops the program before its +1
    println(owners.len())         # never reached

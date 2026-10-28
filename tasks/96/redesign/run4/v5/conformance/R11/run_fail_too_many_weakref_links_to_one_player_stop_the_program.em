#$ test: run-fail
#$ rules: WK-11
#$ count_bits: 8
#$ panics: too many `weakref` links to a `Player` (255)
## design rules: R11
## R11 (verdicts 59, 96): every `weakref` link adds one to its target's weak count, and "before each +1
## the count is compared with its maximum, and at the maximum the program stops". This program is
## compiled with an 8-bit count in each object (`#$ count_bits: 8`), stored and compared as the real
## 32-bit one is, so the maximum is 255: the loop's first 255 rounds make 255 links, which brings the
## count to 255, and round 256's `weakref p` stops the program, where a count that rolled over to 0 would
## let the block go while links still test it. Rules section 2, the stop texts and the paragraph under
## them. Real programs keep the 32-bit count: `R9/run_real_programs_keep_32_bit_counts.em`.

class Player:
    hp: int = 100

fn main():
    p = Player()
    links: Array[weakref Player] = []
    for _ in 0..256:
        links.push(weakref p)     # round 256 stops the program before its +1
    println(links.len())          # never reached

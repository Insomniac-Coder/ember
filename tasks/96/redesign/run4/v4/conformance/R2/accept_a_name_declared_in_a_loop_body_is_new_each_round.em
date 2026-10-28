#$ test: run-pass
#$ rules: R2, R1
#$ stdout: 3
## R2, R1: a line inside a loop's body declares its name once per round (`[GRM-4]`), so `s` is a new
## name in each round and is handed over in each; a name declared before the loop and handed over
## inside it is refused (`R1/reject_an_owned_object_handed_over_in_every_round_of_a_loop`; check-V1's
## R2-15).

fn main():
    v: Array[String] = []
    for _ in 0..3:
        s = String("a")
        v.push(s)
    println(v.len())

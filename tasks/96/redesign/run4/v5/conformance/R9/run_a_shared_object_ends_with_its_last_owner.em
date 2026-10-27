#$ test: run-pass
#$ rules: R9
#$ stdout: consumed 1
#$ stdout: still 1
#$ stdout: end 1
#$ stdout: done
## R9: a `shares` value is "freed when its last owner ends"; its owners are counted with plain adds
## (section 2: "a count +1 / -1, and before each +1 one compare with the count's maximum"). Ending one owner
## of two ends nothing; ending the last one ends the object.

class Token:
    v: int = 1

    fn drop():
        println(f"end {this.v}")

fn consume(owns t: shares Token):
    println(f"consumed {t.v}")    # `t` ends here: the count goes from 2 to 1, the token lives

fn program():
    first = Token()
    second = shares first         # two owners: the count is 2
    consume(first)                # `first` is handed to `consume`
    println(f"still {second.v}")
                                  # `second` ends with `program`: the count goes to 0, the token ends

fn main():
    program()
    println("done")

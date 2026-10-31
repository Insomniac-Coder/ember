#$ test: run-pass
#$ rules: DRP-2, OWN-2
# "[DRP-2]: enum payload of the active variant" — the payload drops with the
# enum, exactly once.

struct R:
    pub n: i32

    fn drop():
        println(this.n)

enum E:
    Hold(R)
    Empty

fn main():
    _e = E.Hold(R(5))
#$ stdout: 5

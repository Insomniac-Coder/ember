#$ test: run-pass
#$ rules: EXP-4, CTL-3b, TXT-10, LT-1, DRP-3
#$ profiles: debug, release, shipping
#$ stdout: a
#$ stdout: b
#$ stdout: drop 9
#$ stdout: loop done
# D-488 (the other agent's case). The nested producer owner must be promoted with
# a source for iterable, remain alive across both items, and end with the loop.

struct Source:
    pub text: String
    pub n: int

    fn drop(mut self):
        println("drop", self.n)

fn viewed(source: Source) -> str:
    return source.text.as_str()

fn main():
    for part in viewed(Source("a,b", 9)).split(","):
        println(part)
    println("loop done")

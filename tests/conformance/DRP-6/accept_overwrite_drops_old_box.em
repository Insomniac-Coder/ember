#$ test: run-pass
#$ rules: HEAP-1, DRP-6, OWN-2, OWN-5
# Ordinary assignment destroys and frees the old Box before the replacement
# becomes the live owner; the final value is destroyed at scope end.

struct Resource:
    pub n: i32

    fn drop():
        println(this.n)

fn main():
    _boxed: owns Resource = owns Resource(1)
    _boxed = owns Resource(2)
    println(0)
#$ stdout: 1
#$ 0
#$ 2
#$ assert-c-count: contains("ember_box_new_copy") == 2
#$ assert-c-count: contains("ember_free(") == 2

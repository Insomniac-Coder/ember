#$ test: run-pass
#$ rules: HEAP-1, DRP-6, OWN-3, BRW-1
# A Box is one move-only heap owner. Field and method access auto-dereference
# without exposing its private raw pointer; get creates an ordinary shared
# reference rooted at the owner.

struct Payload:
    pub n: i32

    fn bump(mut self):
        self.n = self.n + 1

fn main():
    b: Box[Payload] = Box(Payload(40))
    b.bump()
    b.n = b.n + 1
    r: ref Payload = b.get()
    println(r.n)
    b.n = 43
    println(b.n)
#$ stdout: 42
#$ 43
#$ assert-c: contains("ember_box_new_copy")
#$ assert-c: contains("ember_free")
#$ assert-c: contains("typedef em_Payload* em_Box_Payload;")

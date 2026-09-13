#$ test: run-pass
#$ rules: HEAP-1, DRP-6, OWN-2
# `[DRP-6]` fixes the order: first run T's destructor, then free the unique
# allocation. The source output proves the destructor ran; generated-C order
# pins the otherwise invisible relationship to the free.

struct Resource:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    boxed = Box(Resource(7))
    _payload = boxed.get()
    println(0)
#$ stdout: 0
#$ 7
#$ assert-c-order: "em_Resource_drop(&" then "ember_free("

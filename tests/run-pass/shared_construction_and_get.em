#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, OBJ-3
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 7
#$ assert-c: contains(ember_obj_new_copy)
#$ assert-c: contains(ember_retain((ember_obj_header*)
#$ assert-c: contains(ember_release((ember_obj_header*)

struct Token:
    value: i32

    fn drop(mut self):
        println(self.value)

fn main():
    first = Shared(Token(7))
    second = first
    value = second.get()
    println(value.value)

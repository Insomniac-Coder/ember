#$ test: compile-fail
#$ rules: BRW-7
# "No borrow of a moved or uninitialised place. `E3050`." Shape O6.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    r: ref Array[i32] = ref a      #$ error[E3050]: `a` is borrowed after it has been moved out of
    println(r.len())

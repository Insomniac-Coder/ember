#$ test: run-pass
#$ rules: BRW-7, OWN-3
# The other half of `[BRW-7]`: what it forbids is borrowing a place that is
# moved-from or uninitialised, and a place that has been given a new value is
# neither. Definite initialisation is what tracks that, not scope.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    a = Array[i32]()
    a.push(7)
    r: ref Array[i32] = ref a
    println(r.len())
    println(b[0])
#$ stdout: 1
#$ 1

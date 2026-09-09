#$ test: run-pass
#$ rules: OWN-3, OWN-5
# A move ends the old owner's claim; assigning again gives it a new value, and
# the name is usable once more. Definite initialisation is what tracks this,
# not scope.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    a = Array[i32]()
    a.push(2)
    println(b[0])
    println(a[0])
#$ stdout: 1
#$ 2

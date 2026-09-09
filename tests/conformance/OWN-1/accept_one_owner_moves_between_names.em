#$ test: run-pass
#$ rules: OWN-1, OWN-3
# "Every value has exactly one owner." Binding to a second name moves it, so
# the buffer has one owner throughout and is freed once — which the runtime's
# own allocation counter is what proves, not this program's output.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    println(b[0])
#$ stdout: 1

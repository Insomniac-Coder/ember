#$ test: compile-fail
#$ rules: CLO-1, CLO-2, LT-42, OWN-3, TST-19

# Capturing a non-Copy value in `owned fn` moves it into the closure's owned
# environment. The original local is therefore unavailable afterward.

fn main():
    data: Array[i32] = Array[i32]()
    data.push(7)
    task = owned fn() => data[0]
    println(data[0]) #$ error[E3040]: `data` has been moved out of
    println(task())

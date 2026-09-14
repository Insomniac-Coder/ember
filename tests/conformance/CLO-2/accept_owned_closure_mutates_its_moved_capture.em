#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, OWN-3, OWN-5, TST-19
#$ stdout: 1
#$ stdout: 2
#$ assert-c-count: contains("ember_vec_free(") == 1

# `owned fn` owns this Array. Mutating the owned field requires a mutable
# closure call, but does not by itself make the closure `CallableOnce`.

fn main():
    values: Array[i32] = Array[i32]()
    task = owned fn() -> usize:
        values.push(7)
        return values.len()
    println(task())
    println(task())

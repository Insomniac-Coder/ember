#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-3, CLO-4, LT-42, OWN-3, OWN-5, TST-19
#$ stdout: 7
#$ stdout: 7
#$ assert-c-count: contains("ember_vec_free(") == 1

# `owned fn` stores its Array capture by move rather than retaining a borrow of
# `data`. The closure remains directly callable, and the one owning environment
# drops the Array exactly once when it leaves scope.

fn main():
    data: Array[i32] = Array[i32]()
    data.push(7)
    task = owned fn() => data[0]
    println(task())
    println(task())

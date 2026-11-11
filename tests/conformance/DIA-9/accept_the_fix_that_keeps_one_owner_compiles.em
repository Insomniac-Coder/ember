#$ test: run-pass
#$ rules: DIA-9
#$ stdout: 1
# `[DIA-9]` — the fix with one owner and no `unsafe`.

fn append_one(ref mut values: Array[i32]):
    values.push(owns 1)

fn main():
    values: Array[i32] = Array[i32]()
    append_one(ref mut values)
    println(values.len())

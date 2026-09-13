#$ test: run-pass
#$ rules: SPN-2, SPN-3, BRW-2, BRW-5, FN-1a
#$ stdout: 10
#$ stdout: 20
#$ stdout: 30
#$ stdout: 30

# A named MutSpan is reborrowed rather than moved by split_at, and the worked
# expression form remains valid. Once the child views end, both the parent view
# and its Array owner become usable again.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    view = values.as_mut_span()
    parts = view.split_at(1)
    left = parts.0
    right = parts.1
    left[0] = 10
    right[0] = 20
    println(left[0])
    println(right[0])
    view[0] = 30
    println(view[0])

    direct = values.as_mut_span().split_at(2)
    direct_left = direct.0
    println(direct_left[0])

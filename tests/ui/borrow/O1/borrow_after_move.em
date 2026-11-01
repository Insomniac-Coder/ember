fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    moved = values
    println(moved.len())
    view: ref Array[i32] = ref values
    println(view.len())


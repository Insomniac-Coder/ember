fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    view: ref Array[i32] = ref values
    println(view.len())
    moved = values
    println(moved.len())


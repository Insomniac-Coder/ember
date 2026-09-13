fn main():
    value: i32 = 1
    view: ref i32 = ref value
    value = 5
    println(view)


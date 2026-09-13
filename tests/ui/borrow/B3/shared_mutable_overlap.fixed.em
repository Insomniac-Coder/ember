fn main():
    value: i32 = 1
    view: ref i32 = ref value
    println(view)
    value = 5
    println(value)


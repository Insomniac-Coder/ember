pub interface Inspect[T]:
    fn inspect(value: T) -> i32

pub interface Child[T]: needs Inspect[T]:
    fn child(value: T) -> i32

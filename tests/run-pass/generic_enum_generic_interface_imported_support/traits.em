pub interface Inspect[T]:
    fn inspect(self, value: T) -> i32

pub interface Child[T]: Inspect[T]:
    fn child(self, value: T) -> i32

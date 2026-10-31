pub interface Parent[T]:
    fn parent(value: T) -> i32:
        return 20

pub interface Child[T]: needs Parent[T]:
    fn child(value: T) -> i32:
        return 22

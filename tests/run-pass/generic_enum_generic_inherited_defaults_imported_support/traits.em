pub interface Parent[T]:
    fn parent(self, value: T) -> i32:
        return 20

pub interface Child[T]: Parent[T]:
    fn child(self, value: T) -> i32:
        return 22

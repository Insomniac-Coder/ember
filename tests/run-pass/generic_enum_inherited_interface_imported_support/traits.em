pub interface Parent:
    fn parent(self) -> i32

pub interface Child: Parent:
    fn child(self) -> i32

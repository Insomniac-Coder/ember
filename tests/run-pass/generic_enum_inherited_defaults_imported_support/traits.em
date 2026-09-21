pub interface Parent:
    fn parent(self) -> i32:
        return 20

pub interface Child: Parent:
    fn child(self) -> i32:
        return 22

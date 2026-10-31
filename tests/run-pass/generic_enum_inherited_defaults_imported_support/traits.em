pub interface Parent:
    fn parent() -> i32:
        return 20

pub interface Child: needs Parent:
    fn child() -> i32:
        return 22

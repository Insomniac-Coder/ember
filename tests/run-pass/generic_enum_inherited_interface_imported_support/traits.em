pub interface Parent:
    fn parent() -> i32

pub interface Child: needs Parent:
    fn child() -> i32

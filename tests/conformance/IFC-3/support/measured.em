## A parent interface with an associated type, for children in other
## modules (D-458).

pub interface Measured:
    type Unit
    fn measure(self) -> Unit

pub fn touch(c: int) -> int:
    return c

## A child interface in another module than its parent, naming the parent's
## associated type, and a type meeting both (D-458).

from support.measured import Measured

pub interface Weighed: needs Measured:
    fn heaviest() -> Unit

pub struct Crate:
    pub weight: int

extend Crate implements Measured:
    type Unit = int

    fn measure() -> int:
        return this.weight

extend Crate implements Weighed:
    fn heaviest() -> int:
        return this.weight * 3

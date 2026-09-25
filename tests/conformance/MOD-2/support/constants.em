## A type with a public constant and a private one, for
## `reject_a_private_constant_of_a_type.em`.

pub struct Grid:
    pub cells: int

    pub const SIDE: int = 8 * 8
    const SECRET: int = 7 * 6

pub fn secret() -> int:
    return Grid.SECRET

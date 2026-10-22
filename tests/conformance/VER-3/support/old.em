## `[VER-3]` — the deprecated items `VER-3`'s tests use. Their own uses here are not warned about:
## a deprecated item is still its module's to use.

@deprecated(since="0.9", note="use `area_of` instead")
pub fn area(w: int, h: int) -> int:
    return w * h

pub fn area_of(w: int, h: int) -> int:
    return area(w, h)

@deprecated(since="0.8", note="use `Size` instead")
pub struct Dims:
    pub w: int
    pub h: int

pub struct Size:
    pub w: int
    pub h: int

    @deprecated(since="0.9", note="use `area` instead")
    pub fn surface(self) -> int:
        return self.w * self.h

    pub fn area(self) -> int:
        return self.surface()

@deprecated("0.7", "use `LIMIT` instead")
pub const MAX: int = 10

pub const LIMIT: int = MAX

@deprecated(since="0.9", note="use `Size`")
pub enum Shape:
    Square
    Wide

@deprecated(since="0.9", note="use `Measured`")
pub interface Sized2:
    fn size(self) -> int

pub interface Measured:
    fn size(self) -> int

@deprecated(since="0.6", note="count with `LIMIT`")
pub static SEEN: int = 4

@deprecated(since="0.9", note="write `int`")
pub type Count = int

pub struct Maker:
    pub v: int

    @deprecated(since="0.9", note="use `Maker(v)`")
    pub fn make(v: int) -> Maker:
        return Maker(v)

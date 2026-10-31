from std.math import Float

pub interface TextSource:
    type Item: needs (Copy, Display, Debug)
    fn get() -> Item

[S: needs TextSource]
pub struct TextValue:
    pub value: S.Item

    pub fn padded() -> String:
        return f"{this.value:>4}"

[S: needs TextSource]
pub fn text_value(source: S) -> TextValue[S]:
    return TextValue[S](source.get())

pub interface DecimalSource:
    type Item: needs (Float, Display)
    fn get() -> Item

[S: needs DecimalSource]
pub struct DecimalValue:
    pub value: S.Item

[S: needs DecimalSource]
pub fn decimal_value(source: S) -> DecimalValue[S]:
    return DecimalValue[S](source.get())

pub interface OpaqueSource:
    type Item: needs Copy
    fn get() -> Item

[S: needs OpaqueSource]
pub struct OpaqueValue:
    pub value: S.Item

[S: needs OpaqueSource]
pub fn opaque_value(source: S) -> OpaqueValue[S]:
    return OpaqueValue[S](source.get())

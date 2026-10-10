from std.math import Float

pub interface TextSource:
    type Item: Copy + Display + Debug
    fn get(self) -> Item

pub struct TextValue[S: TextSource]:
    pub value: S.Item

    pub fn padded(self) -> String:
        return f"{self.value:>4}"

pub fn text_value[S: TextSource](source: S) -> TextValue[S]:
    return TextValue[S](source.get())

pub interface DecimalSource:
    type Item: Float + Display
    fn get(self) -> Item

pub struct DecimalValue[S: DecimalSource]:
    pub value: S.Item

pub fn decimal_value[S: DecimalSource](source: S) -> DecimalValue[S]:
    return DecimalValue[S](source.get())

pub interface OpaqueSource:
    type Item: Copy
    fn get(self) -> Item

pub struct OpaqueValue[S: OpaqueSource]:
    pub value: S.Item

pub fn opaque_value[S: OpaqueSource](source: S) -> OpaqueValue[S]:
    return OpaqueValue[S](source.get())

## `std.ser.binary` — `[SER-1]`'s binary format, self-describing so that a reader can skip a field
## it does not know. Each value starts with a tag byte: 1 an integer (zigzag LEB128), 2 a float (its
## shortest text, as text is written), 3 a `bool` (one byte), 4 text (its length, then its UTF-8), 5 a
## list (its count, then the elements), 6 a struct (its version, then each field as 9, its name and
## its value, then 7), 8 an enum value (its variant's name, then its payload's fields as a struct's,
## then 7), 10 `None`.

from std.ser import Deserialize, Reader, SerError, Serialize, Writer

## Writes the binary format.
pub struct BinaryWriter:
    out: Array[u8]

    ## It allocates a new `Array`.
    pub fn new() -> BinaryWriter:
        return BinaryWriter([])

    ## The bytes written.
    pub fn bytes(self) -> Span[u8]:
        return self.out.as_span()

    ## It allocates as the `Array` it adds to grows.
    fn varint(mut self, value: u64):
        v = value
        while v >= 128:
            self.out.push(((v % 128) + 128) as u8)
            v = v // 128
        self.out.push(v as u8)

    ## It allocates as the `Array` it adds to grows.
    fn text(mut self, value: str):
        bytes = value.as_bytes()
        self.varint(len(bytes) as u64)
        for b in bytes:
            self.out.push(b)

extend BinaryWriter implements Writer:
    ## It allocates as the `Array` it adds to grows.
    fn begin_struct(mut self, name: str, version: int) -> Result[void, SerError]:
        self.out.push(6)
        self.varint(version as u64)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn field(mut self, name: str) -> Result[void, SerError]:
        self.out.push(9)
        self.text(name)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn end_struct(mut self) -> Result[void, SerError]:
        self.out.push(7)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn begin_variant(mut self, name: str) -> Result[void, SerError]:
        self.out.push(8)
        self.text(name)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn end_variant(mut self) -> Result[void, SerError]:
        self.out.push(7)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn begin_list(mut self, count: int) -> Result[void, SerError]:
        self.out.push(5)
        self.varint(count as u64)
        return Ok(())

    fn end_list(mut self) -> Result[void, SerError]:
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_none(mut self) -> Result[void, SerError]:
        self.out.push(10)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_int(mut self, value: i64) -> Result[void, SerError]:
        self.out.push(1)
        zigzag = (value as u64) * 2 if value >= 0 else ((-(value + 1)) as u64) * 2 + 1
        self.varint(zigzag)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_float(mut self, value: f64) -> Result[void, SerError]:
        self.out.push(2)
        self.text(f"{value}")
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_bool(mut self, value: bool) -> Result[void, SerError]:
        self.out.push(3)
        self.out.push(1 if value else 0)
        return Ok(())

    ## It allocates as the `Array` it adds to grows.
    fn write_str(mut self, value: str) -> Result[void, SerError]:
        self.out.push(4)
        self.text(value)
        return Ok(())

## Reads the binary format.
pub struct BinaryReader:
    data: Array[u8]
    at: int
    ## For each open list: how many elements are left.
    lists: Array[int]

    ## It allocates a new `Array`.
    pub fn new(bytes: Span[u8]) -> BinaryReader:
        data: Array[u8] = []
        for b in bytes:
            data.push(b)
        return BinaryReader(data, 0, [])

    ## It allocates the error's message when the input is malformed.
    fn malformed(self, what: str) -> SerError:
        return SerError.Malformed(f"{what} at byte {self.at}")

    fn peek(self) -> int:
        if self.at < len(self.data):
            return self.data[self.at] as int
        return -1

    ## It allocates the error's message when the input is malformed.
    fn expect(mut self, tag: int, what: str) -> Result[void, SerError]:
        if self.peek() != tag:
            return Err(self.malformed(f"expected {what}"))
        self.at += 1
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn varint(mut self) -> Result[u64, SerError]:
        value: u64 = 0
        scale: u64 = 1
        for _ in 0..10:
            if self.at >= len(self.data):
                return Err(self.malformed("a short number"))
            b = self.data[self.at] as u64
            self.at += 1
            value += (b % 128) * scale
            if b < 128:
                return Ok(value)
            scale = scale * 128
        return Err(self.malformed("a number too long"))

    ## It allocates a new `String`.
    fn text(mut self) -> Result[String, SerError]:
        n = self.varint()? as int
        if self.at + n > len(self.data):
            return Err(self.malformed("short text"))
        match self.data.as_span()[self.at..self.at + n].to_str():
            Ok(s):
                self.at += n
                return Ok(String.from(s))
            Err(_):
                return Err(self.malformed("text that is not UTF-8"))

extend BinaryReader:
    ## A struct's or an enum value's fields, read past, and its end.
    ## It allocates the error's message when the input is malformed.
    fn skip_fields(mut self) -> Result[void, SerError]:
        while self.peek() == 9:
            self.at += 1
            self.text()?
            self.skip_value()?
        return self.expect(7, "the end of a struct")

extend BinaryReader implements Reader:
    ## It allocates the error's message when the input is malformed.
    fn begin_struct(mut self) -> Result[int, SerError]:
        self.expect(6, "a struct")?
        return Ok(self.varint()? as int)

    ## It allocates the error's message when the input is malformed.
    fn next_field(mut self) -> Result[Option[String], SerError]:
        if self.peek() == 7:
            return Ok(None)
        self.expect(9, "a field")?
        return Ok(Some(self.text()?))

    ## It allocates the error's message when the input is malformed.
    fn end_struct(mut self) -> Result[void, SerError]:
        return self.expect(7, "the end of a struct")

    ## It allocates the error's message when the input is malformed.
    fn begin_variant(mut self) -> Result[String, SerError]:
        self.expect(8, "an enum value")?
        return self.text()

    ## It allocates the error's message when the input is malformed.
    fn end_variant(mut self) -> Result[void, SerError]:
        return self.expect(7, "the end of an enum value")

    ## It allocates as the `Array` it adds to grows.
    fn begin_list(mut self) -> Result[void, SerError]:
        self.expect(5, "a list")?
        n = self.varint()? as int
        self.lists.push(n)
        return Ok(())

    ## It allocates the error's message when the input is malformed.
    fn next_element(mut self) -> Result[bool, SerError]:
        n = len(self.lists)
        if n == 0:
            return Err(self.malformed("an element outside a list"))
        if self.lists[n - 1] == 0:
            _ = self.lists.pop()
            return Ok(false)
        self.lists[n - 1] -= 1
        return Ok(true)

    fn read_none(mut self) -> Result[bool, SerError]:
        if self.peek() == 10:
            self.at += 1
            return Ok(true)
        return Ok(false)

    ## It allocates the error's message when the input is malformed.
    fn read_int(mut self) -> Result[i64, SerError]:
        self.expect(1, "an integer")?
        z = self.varint()?
        return Ok((z // 2) as i64 if z % 2 == 0 else -((z // 2) as i64) - 1)

    ## It allocates the error's message when the input is malformed.
    fn read_float(mut self) -> Result[f64, SerError]:
        self.expect(2, "a float")?
        text = self.text()?
        match text.as_str().parse[f64]():
            Ok(x):
                return Ok(x)
            Err(_):
                return Err(self.malformed("a float"))

    ## It allocates the error's message when the input is malformed.
    fn read_bool(mut self) -> Result[bool, SerError]:
        self.expect(3, "a bool")?
        if self.at >= len(self.data):
            return Err(self.malformed("a short bool"))
        b = self.data[self.at]
        self.at += 1
        return Ok(b != 0)

    ## It allocates the error's message when the input is malformed.
    fn read_str(mut self) -> Result[String, SerError]:
        self.expect(4, "text")?
        return self.text()

    ## It allocates a new `String`.
    fn skip_value(mut self) -> Result[void, SerError]:
        tag = self.peek()
        if tag == 1:
            _ = self.read_int()?
        elif tag == 2:
            _ = self.read_float()?
        elif tag == 3:
            _ = self.read_bool()?
        elif tag == 4:
            _ = self.read_str()?
        elif tag == 5:
            self.at += 1
            n = self.varint()?
            for _ in 0..n:
                self.skip_value()?
        elif tag == 6:
            self.at += 1
            self.varint()?
            self.skip_fields()?
        elif tag == 8:
            self.at += 1
            self.text()?
            self.skip_fields()?
        elif tag == 10:
            self.at += 1
        else:
            return Err(self.malformed("an unknown tag"))
        return Ok(())

## `value` in the binary format.
pub fn to_bytes[T: Serialize](value: T) -> Result[Array[u8], SerError]:
    w = BinaryWriter.new()
    value.serialize(w)?
    out: Array[u8] = []
    for b in w.bytes():
        out.push(b)
    return Ok(out)

## A `T` read from the binary format.
pub fn from_bytes[T: Deserialize](bytes: Span[u8]) -> Result[T, SerError]:
    r = BinaryReader.new(bytes)
    return T.deserialize(r)

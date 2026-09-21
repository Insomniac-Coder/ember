@derive(Clone)
pub struct Token:
    pub value: i32

@derive(Clone)
pub enum Entry[T]:
    Value(value: T)

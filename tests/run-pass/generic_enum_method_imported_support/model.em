pub enum Message[T: Eq]:
    Empty

    pub fn choose[U: Eq](self, value: U) -> U:
        return value

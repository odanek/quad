pub struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Self(Vec::new())
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

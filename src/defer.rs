pub struct Defer<F: FnOnce()>(Option<F>);

pub fn defer<F: FnOnce()>(deferred: F) -> Defer<F> {
    Defer(Some(deferred))
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        match self.0.take() {
            Some(f) => f(),
            None => {}, // should never happen
        }
    }
}

pub trait Pohuy: Sized {
    fn pohuy(self) -> () {}
}

impl<T, E> Pohuy for Result<T, E> {}
#[derive(Clone, Copy)]
pub struct Chomp<T>(pub(crate) *const T);

impl<T> Chomp<T> {
    pub fn new(value: &T) -> Self {
        Self(value as *const _)
    }
    pub fn inner(&self) -> T {
        unsafe { std::ptr::read_unaligned(self.0) }
    }
}

pub trait ChompFlatten<T> {
    fn flatten(&self) -> Vec<T>;
}

impl<T> ChompFlatten<T> for Vec<Chomp<T>> {
    fn flatten(&self) -> Vec<T> {
        self.iter().map(|c| c.inner()).collect()
    }
}

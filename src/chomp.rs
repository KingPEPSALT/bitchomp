#[derive(Clone, Copy)]
pub struct Chomp<T>(pub(crate) *const T);

impl<T: Copy> Chomp<T> {
    pub fn new(value: &T) -> Self {
        Self(value as *const _)
    }
    /// # Safety
    /// 
    /// This method reads the value from the raw pointer. Since `T: Copy`,
    /// this is a bitwise copy operation, not a move. The original value
    /// remains valid after this call, preventing double-free UB.
    pub fn inner(&self) -> T {
        unsafe { std::ptr::read_unaligned(self.0) }
    }
}

pub trait ChompFlatten<T: Copy> {
    fn flatten(&self) -> Vec<T>;
}

impl<T: Copy> ChompFlatten<T> for Vec<Chomp<T>> {
    fn flatten(&self) -> Vec<T> {
        self.iter().map(|c| c.inner()).collect()
    }
}

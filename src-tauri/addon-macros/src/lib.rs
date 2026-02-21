pub use addon_macros_impl::addon_guard;

pub trait AddonCheck<T> {
    fn check(&self, requirement: T) -> bool;
}

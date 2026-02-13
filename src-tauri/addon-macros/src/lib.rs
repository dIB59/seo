pub use addon_macros_impl::addon_guard;

pub trait AddonProvider {
    fn verify_addon(&self, addon_name: &str) -> bool;
}

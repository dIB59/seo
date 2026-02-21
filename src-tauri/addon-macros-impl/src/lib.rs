use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, FnArg, ItemFn, Pat};

#[proc_macro_attribute]
pub fn addon_guard(attr: TokenStream, item: TokenStream) -> TokenStream {
    let addon = parse_macro_input!(attr as Expr);
    let mut input = parse_macro_input!(item as ItemFn);

    // Find the argument with the #[provider] attribute
    let mut provider_ident = None;
    for arg in &mut input.sig.inputs {
        if let FnArg::Typed(pat_typed) = arg {
            let mut provider_attr_index = None;
            for (i, attr) in pat_typed.attrs.iter().enumerate() {
                if attr.path().is_ident("provider") {
                    provider_attr_index = Some(i);
                    break;
                }
            }

            if let Some(idx) = provider_attr_index {
                // Remove the #[provider] attribute so the compiler doesn't complain
                pat_typed.attrs.remove(idx);

                if let Pat::Ident(pat_ident) = &*pat_typed.pat {
                    provider_ident = Some(pat_ident.ident.clone());
                    break;
                }
            }
        }
    }

    let state_ident = provider_ident.unwrap_or_else(|| {
        let func_name = &input.sig.ident;
        panic!(
            "addon_guard: Could not find a provider for function '{}'. \
             You MUST mark the provider argument with #[provider]. \
             Example: fn my_command(#[provider] state: State<'_, AppState>)",
            func_name
        )
    });

    let block = &input.block;
    let statements = &block.stmts;

    // Use the AddonCheck trait
    let new_block = quote! {
        {
            use addon_macros::AddonCheck;

            // Check the requirement using the AddonCheck trait
            if !addon_macros::AddonCheck::check(&*#state_ident, #addon) {
                // We'll use a generic error message for now, as the specific error construction
                // depends on the domain types which might not be visible here in a generic way.
                // However, our current implementation expects AddonError::PermissionDenied(#addon).
                // Let's assume the argument implements Debug so we can format it if needed,
                // but for now we follow the pattern of the existing code which likely expects
                // the error type to be available in the scope or we construct it.

                // Inspecting the previous implementation:
                // return Err(crate::domain::licensing::AddonError::FeatureLocked(#addon).into());

                // We will keep this compatible by keeping the assumption that
                // crate::domain::licensing::AddonError is available or we use the fully qualified path.
                return Err(crate::domain::licensing::AddonError::PermissionDenied(#addon).into());
            }
            #(#statements)*
        }
    };

    input.block = syn::parse2(new_block).expect("Failed to parse new block");

    TokenStream::from(quote! {
        #input
    })
}

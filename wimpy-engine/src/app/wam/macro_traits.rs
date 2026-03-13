use super::prelude::*;

// Virtual asset to hard asset
pub trait AssetReferenceResolver {
    fn type_check(asset: &AssetReference) -> Option<&Self>;
}

#[macro_export]
macro_rules! impl_asset_reference_resolver {
    (
        $type:ty,
        $variant:ident
    ) => {
        impl AssetReferenceResolver for $type {
            fn type_check(untyped_asset: &AssetReference) -> Option<&Self> {
                if let AssetReference::$variant(asset) = untyped_asset {
                    Some(asset)
                } else {
                    None
                }
            }
        }
    };
}

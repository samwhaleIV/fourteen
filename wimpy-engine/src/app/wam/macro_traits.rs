
use super::prelude::*;

pub trait AssetReferenceResolver<T> {
    fn type_check(asset: &AssetReference) -> Option<&T>;
}

pub trait HardAssetResolver {
    fn type_check(asset: &HardAsset) -> Option<&Self>;
    fn type_check_mut(asset: &mut HardAsset) -> Option<&mut Self>;
    fn get_type() -> HardAssetType;
}

#[macro_export]
macro_rules! impl_asset_reference_resolver {
    (
        $type:ty,
        $variant:ident
    ) => {
        impl AssetReferenceResolver<Self> for $type {
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

#[macro_export]
macro_rules! impl_hard_asset_resolver {
    (
        $asset_type:ty,
        $variant:ident,
        $type_enum:expr
    ) => {
        impl HardAssetResolver for $asset_type {
            fn type_check(asset: &HardAsset) -> Option<&Self> {
                if let HardAssetData::$variant(data) = &asset.data {
                    Some(data)
                } else {
                    None
                }
            }
            fn type_check_mut(asset: &mut HardAsset) -> Option<&mut Self> {
                if let HardAssetData::$variant(data) = &mut asset.data {
                    Some(data)
                } else {
                    None
                }
            }
            fn get_type() -> HardAssetType {
                $type_enum
            }
        }
    };
}

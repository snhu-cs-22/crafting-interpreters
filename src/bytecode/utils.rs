#[macro_export]
macro_rules! impl_convert_enum_u8 {
    ($enum:ty, $largest_variant:ident) => {
        impl Into<u8> for $enum {
            fn into(self) -> u8 {
                // SAFETY: Because `$enum` is marked `repr(u8)`, all conversions to u8 are valid.
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl TryFrom<u8> for $enum {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, <Self as TryFrom<u8>>::Error> {
                // SAFETY: We assume that the variants in the enum are assigned default values.
                // Because of that assumption, all values up to, and including `$largest_variant`
                // are valid `u8`s and every value greater than `$largest_variant` is invalid.
                if value <= <$enum>::$largest_variant.into() {
                    Ok(unsafe { ::std::mem::transmute(value) })
                } else {
                    Err(())
                }
            }
        }
    }
}

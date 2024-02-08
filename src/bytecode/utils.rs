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

#[macro_export]
macro_rules! impl_wrapper_type {
    ($wrapper:ty, $inner:ty) => {
        impl From<$inner> for $wrapper {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$wrapper> for $inner {
            fn from(value: $wrapper) -> $inner {
                value.0
            }
        }
    }
}

#[macro_export]
macro_rules! impl_binary_ops_for_wrapper_type {
    ($wrapper:ident, $inner:ident, $($op_trait:ident)::*, $op_fn:ident, $op_symbol:tt) => {
        impl $($op_trait)::* for $wrapper {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self {
                Self(self.0 $op_symbol rhs.0)
            }
        }
    }
}

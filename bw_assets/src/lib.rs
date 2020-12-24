#[macro_use]
extern crate derive_builder;

pub mod dat;
pub mod map;
pub mod tbl;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! make_pointer {
        ($name:ident, $type:ty) => {
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct $name(pub(in crate) $type);

            impl From<$name> for usize {
                fn from(p: $name) -> Self {
                    usize::from(&p)
                }
            }

            impl From<&$name> for usize {
                fn from(p: &$name) -> Self {
                    p.0 as usize
                }
            }
        };
    }
}

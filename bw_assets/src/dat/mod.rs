pub use self::image::*;
pub use flingy::*;
pub use sprite::*;
pub use tech_data::*;
pub use unit::*;
pub use upgrade::*;
pub use weapon::*;

mod flingy;
mod image;
mod sprite;
mod tech_data;
mod unit;
mod upgrade;
mod weapon;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! count_total {
        ($block_size:ident) => {
            fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
            where
                I: Clone + PartialEq,
                F: Parser<I, O, E>,
                E: ParseError<I>,
            {
                count(f, $block_size)
            }
        };
    }

    #[macro_export]
    macro_rules! make_dat {
        ($name:ident, $type:ty, $index_type:ty) => {
            #[derive(Debug)]
            pub struct $name(Vec<$type>);

            impl Index<$index_type> for $name {
                type Output = $type;

                fn index(&self, id: $index_type) -> &Self::Output {
                    self.index(&id)
                }
            }

            impl Index<&$index_type> for $name {
                type Output = $type;

                fn index(&self, id: &$index_type) -> &Self::Output {
                    &self.0[usize::from(id)]
                }
            }
        };
    }
}

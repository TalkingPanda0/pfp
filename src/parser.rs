use std::str::FromStr;

use crate::action::Action;

pub trait ParsableAction {
    fn parse_action(input: &str, actions: &mut Vec<Box<dyn Action>>) -> bool;
}

pub trait Parsable: Sized {
    fn parse<'a>(input: &mut impl Iterator<Item = &'a str>) -> Option<Self>;
}

macro_rules! impl_parsable {
    ( $($t:ty),* ) => {
        $(
            impl Parsable for $t {
                fn parse<'a>(input: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
                    let input = input.next()?;
                    <$t as FromStr>::from_str(input).ok()
                }
            }
        )*
    };
}

// Done this way instead of a generic impl to allow the Option<T> impl block
impl_parsable!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char, String
);


impl<T> Parsable for Option<T>
where
    T: Parsable,
{
    fn parse<'a>(input: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
        Some(T::parse(input))
    }
}

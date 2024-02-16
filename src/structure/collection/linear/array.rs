//! [Arrays](https://en.wikipedia.org/wiki/Array_(data_type)).

mod fixed;

use super::Linear;

/// A [`Linear`] [`Collection`] which occupies contigious memory.
pub trait Array<'a>:
    Linear<'a>
    + std::ops::IndexMut<usize>
    + std::ops::DerefMut<Target = [Self::Element]>
    + std::borrow::BorrowMut<[Self::Element]>
    + std::convert::AsMut<[Self::Element]>
    + std::convert::AsRef<[Self::Element]>
{
}

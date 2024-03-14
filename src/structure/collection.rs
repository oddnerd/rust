//! [Collections](https://en.wikipedia.org/wiki/Collection_(abstract_data_type))
//! are data structures which store multiple elements of a single type.

pub mod linear;

/// Multiple instances of a single type (elements) grouped together.
pub trait Collection<'a> {
    /// The type of the elements.
    type Element: 'a;

    /// Query the number of elements.
    fn count(&self) -> usize;

    /// Query if no elements are contained.
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

//! Implementation of [`Singly`].

use super::Collection;
use super::Linear;
use super::List;

/// Independently allocated elements connected via a single link.
///
/// Each element exists within separate allocated object, referred to as a
/// node. Each node contains a single pointer which is said to 'link' to
/// subsequent elements in a [`super::Linear`] sequence. Therefore, [`Self`]
/// points to the first node (if any) and each subsequent node points to the
/// next until the last element which points to nothing as visualized below:
///
/// ```text
///         +-------+    +---------+    +---------+           +------+
/// Self -> | first | -> | element | -> | element | -> ... -> | last |
///         +-------+    +---------+    +---------+           +------+
/// ```
///
/// See also: [Wikipedia](https://en.wikipedia.org/wiki/Linked_list)
pub struct Singly<T> {
    /// The contained elements.
    elements: Option<Box<Node<T>>>,
}

/// An independently allocated element contained within some [`Singly`].
struct Node<T> {
    /// The underlying contained value.
    element: T,

    /// Link to the rest of the list.
    next: Option<Box<Node<T>>>,
}

impl<T> Drop for Singly<T> {
    /// Iteratively drop all contained elements.
    ///
    /// The default destructor implementation will _NOT_ be tail recursive,
    /// this means it will not optimize to an iterative implementation, hence
    /// it could overflow the call stack if enough elements are contained.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::<()>::default();
    ///
    /// for _ in 0..i16::MAX {
    ///     instance.prepend(());
    /// }
    ///
    /// // This would overflow the call stack if not for the implementation.
    /// drop(instance);
    /// ```
    fn drop(&mut self) {
        for element in self {
            drop(element);
        }
    }
}

impl<T> Default for Singly<T> {
    /// Create an empty instance of [`Singly`].
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let instance = Singly::<usize>::default();
    ///
    /// assert_eq!(instance.len(), 0);
    /// ```
    fn default() -> Self {
        Singly { elements: None }
    }
}

impl<T: Clone> Clone for Singly<T> {
    /// Clone all contained elements into a new instance of [`Singly`].
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let original = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// let clone = original.clone();
    ///
    /// assert_eq!(clone, original);
    /// ```
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }
}

impl<T: PartialEq> PartialEq for Singly<T> {
    /// Query if `other` has the same elements in the same order.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let elements = [0, 1, 2, 3, 4, 5];
    ///
    /// let first = Singly::from_iter(elements.iter().copied());
    /// let second = Singly::from_iter(elements.iter().copied());
    ///
    /// assert_eq!(first, second);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for Singly<T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for Singly<T> {
    /// List the elements contained.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Singly::from_iter(expected.iter().copied());
    ///
    /// assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    /// ```
    fn fmt(&self, output: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        output.debug_list().entries(self.iter()).finish()
    }
}

impl<T> core::ops::Index<usize> for Singly<T> {
    type Output = T;

    /// Obtain an immutable reference to the element at position `index`.
    ///
    /// # Panics
    /// This method has the precondition that the `index` is within bounds.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Singly::from_iter(expected.iter().copied());
    ///
    /// for index in 0..expected.len() {
    ///     use core::ops::Index;
    ///     assert_eq!(actual.index(index), expected.index(index));
    /// }
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        let mut next = self.elements.as_deref();

        for _ in 0..index {
            if let Some(current) = next {
                next = current.next.as_deref();
            } else {
                break;
            }
        }

        next.map_or_else(|| panic!("index out of bounds"), |node| &node.element)
    }
}

impl<T> core::ops::IndexMut<usize> for Singly<T> {
    /// Obtain a mutable reference to the element at position `index`.
    ///
    /// # Panics
    /// This method has the precondition that the `index` is within bounds.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let mut actual = Singly::from_iter(expected.iter().copied());
    ///
    /// for index in 0..expected.len() {
    ///     use core::ops::IndexMut;
    ///     assert_eq!(actual.index_mut(index), expected.index_mut(index));
    /// }
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let mut next = self.elements.as_deref_mut();

        for _ in 0..index {
            if let Some(current) = next {
                next = current.next.as_deref_mut();
            } else {
                break;
            }
        }

        next.map_or_else(|| panic!("index out of bounds"), |node| &mut node.element)
    }
}

impl<T> Iterator for Singly<T> {
    type Item = T;

    /// Obtain the first element by value via moving it out of [`Self`].
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
    ///
    /// assert_eq!(instance.next(), Some(0));
    /// assert_eq!(instance.next(), Some(1));
    /// assert_eq!(instance.next(), Some(2));
    /// assert_eq!(instance.next(), Some(3));
    /// assert_eq!(instance.next(), Some(4));
    /// assert_eq!(instance.next(), Some(5));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        let removed = self.elements.take()?;

        self.elements = removed.next;

        Some(removed.element)
    }

    /// Query how many elements are contained.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let instance = Singly::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
    ///
    /// assert_eq!(instance.size_hint(), (6, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.count();

        (count, Some(count))
    }
}

impl<T> DoubleEndedIterator for Singly<T> {
    /// Obtain the last element by value via moving it out of [`Self`].
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
    ///
    /// assert_eq!(instance.next_back(), Some(5));
    /// assert_eq!(instance.next_back(), Some(4));
    /// assert_eq!(instance.next_back(), Some(3));
    /// assert_eq!(instance.next_back(), Some(2));
    /// assert_eq!(instance.next_back(), Some(1));
    /// assert_eq!(instance.next_back(), Some(0));
    /// assert_eq!(instance.next_back(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut removed = self.elements.take()?;
        let mut successor = removed.next.take();
        let mut predecessor = &mut self.elements;

        while let Some(mut current) = successor.take() {
            successor = current.next.take();
            predecessor = &mut predecessor.insert(removed).next;
            removed = current;
        }

        Some(removed.element)
    }
}

impl<T> ExactSizeIterator for Singly<T> {}

impl<T> core::iter::FusedIterator for Singly<T> {}

impl<T> Extend<T> for Singly<T> {
    /// Append the `elements` at the end, maintaining order.
    ///
    /// Using [`Self::append`] directly would be O(N^2) since it is required to
    /// traverse all existing elements for each insertion whereas this method
    /// maintains a pointer to the last element thereby being more efficient.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2]);
    ///
    /// instance.extend([3, 4, 5]);
    ///
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, elements: Iter) {
        let mut current = &mut self.elements;

        while let &mut Some(ref mut next) = current {
            current = &mut next.next;
        }

        for element in elements {
            let element = Box::new(Node {
                element,
                next: None,
            });

            current = &mut current.insert(element).next;
        }
    }
}

impl<T> FromIterator<T> for Singly<T> {
    /// Construct an instance with `elements`.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Singly::from_iter(expected.iter().copied());
    ///
    /// assert!(actual.eq(expected));
    /// ```
    fn from_iter<Iter: IntoIterator<Item = T>>(elements: Iter) -> Self {
        let mut instance = Singly::<T>::default();

        instance.extend(elements);

        instance
    }
}

impl<T> Collection for Singly<T> {
    type Element = T;

    /// Query how many elements are contained.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::Collection;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.count(), 6);
    /// ```
    fn count(&self) -> usize {
        let mut count: usize = 0;

        let mut next = &self.elements;

        while let Some(current) = next.as_deref() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more elements than supported by the address space (usize::MAX)");
            }

            next = &current.next;
        }

        count
    }
}

impl<T> Linear for Singly<T> {
    /// Iterate over the elements by immutable reference.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let instance = Singly::from_iter(expected.iter().copied());
    ///
    /// assert!(instance.iter().eq(expected.iter()));
    /// ```
    fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Self::Element> + ExactSizeIterator + core::iter::FusedIterator
    {
        Iter {
            next: &self.elements,
            previous_back: core::ptr::null(),
        }
    }

    /// Iterate over the elements by mutable reference.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let mut instance = Singly::from_iter(expected.iter().copied());
    ///
    /// assert!(instance.iter_mut().eq(expected.iter_mut()));
    /// ```
    fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut Self::Element>
    + ExactSizeIterator
    + core::iter::FusedIterator {
        IterMut {
            next: self.elements.as_deref_mut(),
            previous_back: core::ptr::null(),
        }
    }

    /// Obtain an immutable reference to the element at position `index`.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(actual.at(0), Some(&0));
    /// assert_eq!(actual.at(1), Some(&1));
    /// assert_eq!(actual.at(2), Some(&2));
    /// assert_eq!(actual.at(3), Some(&3));
    /// assert_eq!(actual.at(4), Some(&4));
    /// assert_eq!(actual.at(5), Some(&5));
    /// assert_eq!(actual.at(6), None);
    /// ```
    fn at(&self, index: usize) -> Option<&Self::Element> {
        let mut next = self.elements.as_deref();

        for _ in 0..index {
            if let Some(current) = next {
                next = current.next.as_deref();
            } else {
                break;
            }
        }

        next.map(|node| &node.element)
    }

    /// Obtain a mutable reference to the element at position `index`.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(actual.at_mut(0), Some(&mut 0));
    /// assert_eq!(actual.at_mut(1), Some(&mut 1));
    /// assert_eq!(actual.at_mut(2), Some(&mut 2));
    /// assert_eq!(actual.at_mut(3), Some(&mut 3));
    /// assert_eq!(actual.at_mut(4), Some(&mut 4));
    /// assert_eq!(actual.at_mut(5), Some(&mut 5));
    /// assert_eq!(actual.at_mut(6), None);
    /// ```
    fn at_mut(&mut self, index: usize) -> Option<&mut Self::Element> {
        let mut next = self.elements.as_deref_mut();

        for _ in 0..index {
            if let Some(current) = next {
                next = current.next.as_deref_mut();
            } else {
                break;
            }
        }

        next.map(|node| &mut node.element)
    }

    /// Query the element considered to be at the front, the first element.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(actual.first(), Some(&0));
    /// ```
    fn first(&self) -> Option<&Self::Element> {
        self.elements.as_deref().map(|node| &node.element)
    }

    /// Query the element considered to be at the back, the last element.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(Linear::last(&actual), Some(&5));
    /// ```
    fn last(&self) -> Option<&Self::Element> {
        let mut current = self.elements.as_deref()?;

        while let Some(next) = current.next.as_deref() {
            current = next;
        }

        Some(&current.element)
    }

    /// Obtain a reference to the element at the front, the first element.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(actual.first_mut(), Some(&mut 0));
    /// ```
    fn first_mut(&mut self) -> Option<&mut Self::Element> {
        self.elements.as_deref_mut().map(|node| &mut node.element)
    }

    /// Obtain a reference to the element at the back, the last element.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(actual.last_mut(), Some(&mut 5));
    /// ```
    fn last_mut(&mut self) -> Option<&mut Self::Element> {
        let mut current = self.elements.as_deref_mut()?;

        while let Some(next) = current.next.as_deref_mut() {
            current = next;
        }

        Some(&mut current.element)
    }
}

impl<T> List for Singly<T> {
    /// Move an `element` into such that it becomes the element at `index`.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This method takes O(N) times and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 4, 5]);
    ///
    /// assert!(instance.insert(3, 3).is_ok_and(|inserted| inserted == &3));
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn insert(
        &mut self,
        index: usize,
        element: Self::Element,
    ) -> Result<&mut Self::Element, Self::Element> {
        let mut next = &mut self.elements;

        for _ in 0..index {
            if let &mut Some(ref mut current) = next {
                next = &mut current.next;
            } else {
                return Err(element);
            }
        }

        let new = Box::new(Node {
            element,
            next: next.take(),
        });

        Ok(&mut next.insert(new).element)
    }

    /// Move the element at `index` out, if it exists.
    ///
    /// # Performance
    /// This method takes O(N) times and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert!(instance.remove(3).is_some_and(|inserted| inserted == 3));
    /// assert!(instance.eq([0, 1, 2, 4, 5]));
    /// ```
    fn remove(&mut self, index: usize) -> Option<Self::Element> {
        let mut next = &mut self.elements;

        for _ in 0..index {
            if let &mut Some(ref mut current) = next {
                next = &mut current.next;
            } else {
                return None;
            }
        }

        next.take().map(|removed| {
            *next = removed.next;

            removed.element
        })
    }

    /// Move an `element` into a new node at the front to become the first.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([1, 2, 3, 4, 5]);
    ///
    /// instance.prepend(0);
    ///
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn prepend(&mut self, element: Self::Element) -> Result<&mut Self::Element, Self::Element> {
        let new = Box::new(Node {
            element,
            next: self.elements.take(),
        });

        let new = self.elements.insert(new);

        Ok(&mut new.element)
    }

    /// Move an `element` into a new node at the back to become the last.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4]);
    ///
    /// instance.append(5);
    ///
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn append(&mut self, element: Self::Element) -> Result<&mut Self::Element, Self::Element> {
        let mut next = &mut self.elements;

        while let &mut Some(ref mut current) = next {
            next = &mut current.next;
        }

        let new = Box::new(Node {
            element,
            next: None,
        });

        let new = next.insert(new);

        Ok(&mut new.element)
    }

    /// Remove the element at the front, the first element, if any.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.front(), Some(0));
    /// assert_eq!(instance.front(), Some(1));
    /// assert_eq!(instance.front(), Some(2));
    /// assert_eq!(instance.front(), Some(3));
    /// assert_eq!(instance.front(), Some(4));
    /// assert_eq!(instance.front(), Some(5));
    /// assert_eq!(instance.front(), None);
    /// ```
    fn front(&mut self) -> Option<Self::Element> {
        let mut removed = self.elements.take()?;

        self.elements = removed.next.take();

        Some(removed.element)
    }

    /// Remove the element at the back, the last element, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.back(), Some(5));
    /// assert_eq!(instance.back(), Some(4));
    /// assert_eq!(instance.back(), Some(3));
    /// assert_eq!(instance.back(), Some(2));
    /// assert_eq!(instance.back(), Some(1));
    /// assert_eq!(instance.back(), Some(0));
    /// assert_eq!(instance.back(), None);
    /// ```
    fn back(&mut self) -> Option<Self::Element> {
        let mut next = &mut self.elements;

        while let Some(current) = next.take() {
            if current.next.is_some() {
                let current = next.insert(current);
                next = &mut current.next;
            } else {
                return Some(current.element);
            }
        }

        None
    }

    /// Efficiently remove the elements within the given index `range`.
    ///
    /// Using [`Self::remove`] would be inefficient because each removal would
    /// require traversing the list to the given index which is O(N^2) time,
    /// whereas this method traverses the list only once thereby being O(N).
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert!(instance.drain(1..=4).eq([1, 2, 3, 4]));
    ///
    /// assert!(instance.eq([0, 5]));
    /// ```
    fn drain(
        &mut self,
        range: impl core::ops::RangeBounds<usize>,
    ) -> impl DoubleEndedIterator<Item = Self::Element> + ExactSizeIterator {
        let (offset, remaining) = (|| {
            // This may be more than the number of elements contained.
            let offset = match range.start_bound() {
                core::ops::Bound::Included(start) => *start,
                core::ops::Bound::Excluded(start) => {
                    if let Some(incremented) = start.checked_add(1) {
                        incremented
                    } else {
                        return (0, 0);
                    }
                }
                core::ops::Bound::Unbounded => 0,
            };

            // This may be more than the number of elements after next.
            let remaining = match range.end_bound() {
                core::ops::Bound::Included(end) => end.abs_diff(offset).saturating_add(1),
                core::ops::Bound::Excluded(end) => end.abs_diff(offset),
                core::ops::Bound::Unbounded => usize::MAX.abs_diff(offset),
            };

            (offset, remaining)
        })();

        let mut next = &mut self.elements;

        for _ in 0..offset {
            if let &mut Some(ref mut current) = next {
                next = &mut current.next;
            } else {
                break;
            }
        }

        Drain { next, remaining }
    }

    /// Remove elements matching some `predicate`.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert!(instance.withdraw(|element| element % 2 == 0).eq([0, 2, 4]));
    ///
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn withdraw(
        &mut self,
        predicate: impl FnMut(&Self::Element) -> bool,
    ) -> impl DoubleEndedIterator<Item = Self::Element> {
        Withdraw {
            next: &mut self.elements,
            previous_back: core::ptr::null(),
            predicate,
        }
    }
}

impl<T> super::super::Stack for Singly<T> {
    /// Move an `element` on the top of the stack.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::<usize>::default();
    ///
    /// instance.push(5).expect("successful allocation");
    /// instance.push(4).expect("successful allocation");
    /// instance.push(3).expect("successful allocation");
    /// instance.push(2).expect("successful allocation");
    /// instance.push(1).expect("successful allocation");
    /// instance.push(0).expect("successful allocation");
    ///
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn push(&mut self, element: Self::Element) -> Result<&mut Self::Element, Self::Element> {
        self.prepend(element)
    }

    /// Move out the element at the top of the stack.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.pop(), Some(0));
    /// assert_eq!(instance.pop(), Some(1));
    /// assert_eq!(instance.pop(), Some(2));
    /// assert_eq!(instance.pop(), Some(3));
    /// assert_eq!(instance.pop(), Some(4));
    /// assert_eq!(instance.pop(), Some(5));
    /// assert_eq!(instance.pop(), None);
    /// ```
    fn pop(&mut self) -> Option<Self::Element> {
        self.front()
    }

    /// Query the element at the top of the stack.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.peek(), Some(&0));
    /// ```
    fn peek(&self) -> Option<&Self::Element> {
        self.first()
    }
}

impl<T> super::super::Queue for Singly<T> {
    /// Move an `element` to the end of the queue.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Queue;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::<usize>::default();
    ///
    /// instance.push(0).expect("successful allocation");
    /// instance.push(1).expect("successful allocation");
    /// instance.push(2).expect("successful allocation");
    /// instance.push(3).expect("successful allocation");
    /// instance.push(4).expect("successful allocation");
    /// instance.push(5).expect("successful allocation");
    ///
    /// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
    /// ```
    fn push(&mut self, element: Self::Element) -> Result<&mut Self::Element, Self::Element> {
        self.append(element)
    }

    /// Move out the element at the front of the queue.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Queue;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.pop(), Some(0));
    /// assert_eq!(instance.pop(), Some(1));
    /// assert_eq!(instance.pop(), Some(2));
    /// assert_eq!(instance.pop(), Some(3));
    /// assert_eq!(instance.pop(), Some(4));
    /// assert_eq!(instance.pop(), Some(5));
    /// assert_eq!(instance.pop(), None);
    /// ```
    fn pop(&mut self) -> Option<Self::Element> {
        self.front()
    }

    /// Query the element at the front of the queue.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Queue;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.peek(), Some(&0));
    /// ```
    fn peek(&self) -> Option<&Self::Element> {
        self.first()
    }
}

/// Immutable iterator over a [`Singly`].
struct Iter<'a, T> {
    /// The next element to yield, if any.
    next: &'a Option<Box<Node<T>>>,

    /// The previously yielded element from the back, if any.
    previous_back: *const Node<T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    /// Obtain the next element from the front, if any.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.iter();
    ///
    /// assert_eq!(instance.next(), Some(&0));
    /// assert_eq!(instance.next(), Some(&1));
    /// assert_eq!(instance.next(), Some(&2));
    /// assert_eq!(instance.next(), Some(&3));
    /// assert_eq!(instance.next(), Some(&4));
    /// assert_eq!(instance.next(), Some(&5));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        self.next.as_deref().and_then(|current| {
            if core::ptr::addr_eq(current, self.previous_back) {
                None
            } else {
                self.next = &current.next;
                Some(&current.element)
            }
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let instance = underlying.iter_mut();
    ///
    /// assert_eq!(instance.size_hint(), (6, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count: usize = 0;

        let mut next = self.next;

        while let Some(current) = next.as_deref() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more elements than supported by the address space (usize::MAX)");
            }

            next = &current.next;
        }

        (count, Some(count))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    /// Obtain the next element from the back, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.iter().rev();
    ///
    /// assert_eq!(instance.next(), Some(&5));
    /// assert_eq!(instance.next(), Some(&4));
    /// assert_eq!(instance.next(), Some(&3));
    /// assert_eq!(instance.next(), Some(&2));
    /// assert_eq!(instance.next(), Some(&1));
    /// assert_eq!(instance.next(), Some(&0));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = self.next.as_deref()?;

        if core::ptr::addr_eq(current, self.previous_back) {
            return None;
        }

        while let Some(next) = current.next.as_deref() {
            if core::ptr::addr_eq(next, self.previous_back) {
                break;
            }

            current = next;
        }

        self.previous_back = current;

        Some(&current.element)
    }
}

impl<'a, T: 'a> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for Iter<'a, T> {}

/// Mutable iterator over a [`Singly`].
struct IterMut<'a, T> {
    /// The next element to yield, if any.
    next: Option<&'a mut Node<T>>,

    /// The previously yielded element from the back, if any.
    previous_back: *const Node<T>,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    /// Obtain the next element from the front, if any.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.iter_mut();
    ///
    /// assert_eq!(instance.next(), Some(&mut 0));
    /// assert_eq!(instance.next(), Some(&mut 1));
    /// assert_eq!(instance.next(), Some(&mut 2));
    /// assert_eq!(instance.next(), Some(&mut 3));
    /// assert_eq!(instance.next(), Some(&mut 4));
    /// assert_eq!(instance.next(), Some(&mut 5));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().and_then(|current| {
            if core::ptr::addr_eq(current, self.previous_back) {
                None
            } else {
                self.next = current.next.as_deref_mut();
                Some(&mut current.element)
            }
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 4, 4, 5]);
    /// let instance = underlying.iter_mut();
    ///
    /// assert_eq!(instance.size_hint(), (6, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count: usize = 0;

        let mut next = self.next.as_deref();

        while let Some(current) = next.take() {
            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more elements than supported by the address space (usize::MAX)");
            }

            next = current.next.as_deref();
        }

        (count, Some(count))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    /// Obtain the next element from the back, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.iter_mut();
    ///
    /// assert_eq!(instance.next_back(), Some(&mut 5));
    /// assert_eq!(instance.next_back(), Some(&mut 4));
    /// assert_eq!(instance.next_back(), Some(&mut 3));
    /// assert_eq!(instance.next_back(), Some(&mut 2));
    /// assert_eq!(instance.next_back(), Some(&mut 1));
    /// assert_eq!(instance.next_back(), Some(&mut 0));
    /// assert_eq!(instance.next_back(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut current = self.next.as_deref_mut()?;
        let mut ptr = core::ptr::from_mut(current);

        if core::ptr::addr_eq(current, self.previous_back) {
            return None;
        }

        while let Some(next) = current.next.as_deref_mut() {
            if core::ptr::addr_eq(next, self.previous_back) {
                break;
            }

            current = next;
            ptr = core::ptr::from_mut(current);
        }

        self.previous_back = ptr;

        let element = {
            let element = core::ptr::from_mut(&mut current.element);

            // SAFETY:
            // This is probably undefined behaviour, but it is the best I got.
            //
            // Since the reference to the next node has mutable access to all
            // subsequent nodes, the borrow checker cannot statically guarantee
            // it will not be advanced to access nodes already yielded from the
            // back via this method thereby creating aliasing mutable
            // references (undefined behaviour). In effect, we are consuming
            // the internal mutable reference to create a mutable reference to
            // the back node. However, because we explicitly prevent advancing
            // the internal mutable reference to nodes whose elements have
            // already been yielded, therefore we prevent creating aliasing
            // references. Moreover, because we have mutable access to the
            // entirety of nodes whilst only ever yielding mutable references
            // to contained elements, we can be sure the mutable access to the
            // next front node will not invalidate the lifetime of references
            // to elements at the back.
            unsafe { &mut *element }
        };

        Some(element)
    }
}

impl<'a, T: 'a> ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for IterMut<'a, T> {}

/// [`Iterator`] to yield elements within an index range from [`Singly`].
///
/// See [`Singly::drain`].
struct Drain<'a, T> {
    /// The next element from the front to be yielded, if any.
    next: &'a mut Option<Box<Node<T>>>,

    /// The number of elements yet to be yielded.
    remaining: usize,
}

impl<'a, T: 'a> Drop for Drain<'a, T> {
    /// Remove elements yet to be drained and repair the underlying [`Singly`].
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// core::mem::drop(instance.drain(1..=4));
    ///
    /// assert!(instance.eq([0, 5]));
    /// ```
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<'a, T: 'a> Iterator for Drain<'a, T> {
    type Item = T;

    /// Obtain the next element from the front, if any exists.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.drain(1..=4);
    ///
    /// assert_eq!(instance.next(), Some(1));
    /// assert_eq!(instance.next(), Some(2));
    /// assert_eq!(instance.next(), Some(3));
    /// assert_eq!(instance.next(), Some(4));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.checked_sub(1).and_then(|decremented| {
            self.remaining = decremented;

            let removed = self.next.take()?;

            *self.next = removed.next;

            Some(removed.element)
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let instance = underlying.drain(1..=4);
    ///
    /// assert_eq!(instance.size_hint(), (4, Some(4)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count: usize = 0;
        let mut remaining = self.remaining;

        let mut next = self.next.as_deref();

        while let Some(current) = next.take() {
            if let Some(decremented) = remaining.checked_sub(1) {
                remaining = decremented;
            } else {
                break;
            }

            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more elements than supported by the address space (usize::MAX)");
            }

            next = current.next.as_deref();
        }

        (count, Some(count))
    }
}

impl<'a, T: 'a> DoubleEndedIterator for Drain<'a, T> {
    /// Obtain the next element from the back, if any exists.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut instance = underlying.drain(1..=4);
    ///
    /// assert_eq!(instance.next_back(), Some(4));
    /// assert_eq!(instance.next_back(), Some(3));
    /// assert_eq!(instance.next_back(), Some(2));
    /// assert_eq!(instance.next_back(), Some(1));
    /// assert_eq!(instance.next_back(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        self.remaining.checked_sub(1).and_then(|decremented| {
            self.remaining = decremented;

            let mut removed = self.next.take()?;
            let mut successor = removed.next.take();
            let mut predecessor = &mut *self.next;

            for _ in 0..self.remaining {
                if let Some(mut current) = successor.take() {
                    successor = current.next.take();
                    predecessor = &mut predecessor.insert(removed).next;
                    removed = current;
                } else {
                    break;
                }
            }

            *predecessor = successor;

            Some(removed.element)
        })
    }
}

impl<'a, T: 'a> ExactSizeIterator for Drain<'a, T> {}

impl<'a, T: 'a> core::iter::FusedIterator for Drain<'a, T> {}

/// [`Iterator`] to yield elements matching a predicate from [`Singly`].
///
/// See [`Singly::withdraw`].
struct Withdraw<'a, T, F: FnMut(&T) -> bool> {
    /// The next element to query with the predicate, if any.
    next: &'a mut Option<Box<Node<T>>>,

    /// The previously yielded element from the back, if any.
    previous_back: *const Node<T>,

    /// The predicate based upon which elements are withdrawn.
    predicate: F,
}

impl<T, F: FnMut(&T) -> bool> Drop for Withdraw<'_, T, F> {
    /// Drop elements yet to be yielded and repair the underlying [`Singly`].
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// drop(instance.withdraw(|element| element % 2 == 0));
    ///
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<T, F: FnMut(&T) -> bool> Iterator for Withdraw<'_, T, F> {
    type Item = T;

    /// Obtain the next elements matching the predicate, if any.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert!(instance.withdraw(|element| element % 2 == 0).eq([0, 2, 4]));
    ///
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        let mut removed = self.next.take()?;
        let mut successor = removed.next.take();
        let mut predecessor = &mut *self.next;

        if core::ptr::addr_eq(&*removed, self.previous_back) {
            let inserted = predecessor.insert(removed);
            inserted.next = successor;

            return None;
        }

        if (self.predicate)(&removed.element) {
            *predecessor = successor;

            return Some(removed.element);
        }

        while let Some(mut current) = successor.take() {
            if core::ptr::addr_eq(&*current, self.previous_back) {
                successor = Some(current);

                break;
            }

            if (self.predicate)(&current.element) {
                let inserted = predecessor.insert(removed);
                inserted.next = current.next.take();

                // SAFETY: node will outlive the lifetime of this iterator.
                self.next = unsafe { &mut *core::ptr::from_mut(&mut inserted.next) };

                return Some(current.element);
            }

            successor = current.next.take();
            predecessor = &mut predecessor.insert(removed).next;
            removed = current;
        }

        let inserted = predecessor.insert(removed);
        inserted.next = successor;

        // SAFETY: node will outlive the lifetime of this iterator.
        self.next = unsafe { &mut *core::ptr::from_mut(&mut inserted.next) };

        None
    }

    /// Query how many elements could be yielded.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// let instance = underlying.withdraw(|element| element % 2 == 0);
    ///
    /// assert_eq!(instance.size_hint(), (0, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count: usize = 0;

        let mut next = &*self.next;

        while let Some(current) = next.as_deref() {
            if core::ptr::addr_eq(current, self.previous_back) {
                break;
            }

            if let Some(incremented) = count.checked_add(1) {
                count = incremented;
            } else {
                unreachable!("more elements than supported by the address space (usize::MAX)");
            }

            next = &current.next;
        }

        (0, Some(count))
    }
}

impl<T, F: FnMut(&T) -> bool> DoubleEndedIterator for Withdraw<'_, T, F> {
    /// Obtain the last element matching the predicate, if any.
    ///
    /// # Performance
    /// This method takes O(N^2) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::list::Singly;
    ///
    /// let mut instance = Singly::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert!(instance.withdraw(|element| element % 2 == 0).rev().eq([4, 2, 0]));
    ///
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(mut removed) = self.next.take() {
            if core::ptr::addr_eq(&*removed, self.previous_back) {
                *self.next = Some(removed);
                break;
            }

            let mut successor = removed.next.take();
            let mut predecessor = &mut *self.next;

            while let Some(mut current) = successor.take() {
                if core::ptr::addr_eq(&*current, self.previous_back) {
                    successor = Some(current);

                    break;
                }

                successor = current.next.take();
                predecessor = &mut predecessor.insert(removed).next;
                removed = current;
            }

            if (self.predicate)(&removed.element) {
                *predecessor = successor;

                return Some(removed.element);
            }

            let inserted = predecessor.insert(removed);
            inserted.next = successor;

            self.previous_back = &**inserted;
        }

        None
    }
}

impl<T, F: FnMut(&T) -> bool> ExactSizeIterator for Withdraw<'_, T, F> {}

impl<T, F: FnMut(&T) -> bool> core::iter::FusedIterator for Withdraw<'_, T, F> {}

#[cfg(test)]
mod test {
    use super::*;

    mod drop {
        use super::*;

        #[test]
        fn empty() {
            let instance = Singly::<usize>::default();

            drop(instance);
        }

        #[test]
        fn zero_size_type() {
            let instance: Singly<_> = [(), (), (), (), (), ()].into_iter().collect();

            drop(instance);
        }

        #[test]
        fn drops_elements() {
            use crate::test::mock::DropCounter;

            const ELEMENTS: usize = 256;

            let dropped = DropCounter::new_counter();

            let mut actual = Singly::<DropCounter>::default();

            for _ in 0..ELEMENTS {
                _ = actual
                    .append(DropCounter::new(&dropped))
                    .expect("successful allocation");
            }

            drop(actual);

            assert_eq!(dropped.take(), ELEMENTS);
        }
    }

    mod default {
        use super::*;

        #[test]
        fn has_no_elements() {
            let instance = Singly::<()>::default();

            assert!(instance.elements.is_none());
        }
    }

    mod clone {
        use super::*;

        #[test]
        fn has_elements() {
            let expected = Singly::from_iter([0, 1, 2, 3, 4, 5]);

            let actual = expected.clone();

            assert_eq!(actual.len(), expected.len());
        }

        #[test]
        fn is_equivalent() {
            let expected = Singly::from_iter([0, 1, 2, 3, 4, 5]);

            let actual = expected.clone();

            assert_eq!(actual, expected);
        }

        #[test]
        fn owns_elements() {
            let expected = Singly::from_iter([0, 1, 2, 3, 4, 5]);

            let actual = expected.clone();

            for (clone, original) in actual.iter().zip(expected.iter()) {
                assert!(!core::ptr::addr_eq(clone, original));
            }
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn eq_when_same_elements() {
            let elements = [0, 1, 2, 3, 4, 5];

            let first: Singly<_> = elements.iter().copied().collect();
            let second: Singly<_> = elements.iter().copied().collect();

            assert_eq!(first, second);
        }

        #[test]
        fn ne_when_different_elements() {
            let first: Singly<_> = [0].into_iter().collect();
            let second: Singly<_> = [1].into_iter().collect();

            assert_ne!(first, second);
        }

        #[test]
        fn is_symmetric() {
            let elements = [0, 1, 2, 3, 4, 5];

            let first: Singly<_> = elements.iter().copied().collect();
            let second: Singly<_> = elements.iter().copied().collect();

            // `first == second` <=> `second == first`
            assert_eq!(first, second);
            assert_eq!(second, first);
        }

        #[test]
        fn is_transitive() {
            let elements = [0, 1, 2, 3, 4, 5];

            let first: Singly<_> = elements.iter().copied().collect();
            let second: Singly<_> = elements.iter().copied().collect();
            let third: Singly<_> = elements.iter().copied().collect();

            // `first == second && second == third` => `first == third`
            assert_eq!(first, second);
            assert_eq!(second, third);
            assert_eq!(third, first);
        }

        #[test]
        fn is_reflexive() {
            let actual = Singly::<()>::default();

            assert_eq!(actual, actual);
        }
    }

    mod fmt {
        use super::*;

        mod debug {
            use super::*;

            #[test]
            fn is_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
            }
        }
    }

    mod index {
        use super::*;

        use core::ops::Index as _;

        #[test]
        fn correct_element() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Singly::from_iter(expected);

            for (index, value) in expected.iter().enumerate() {
                assert_eq!(actual.index(index), value);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let instance = Singly::<()>::default();

            let _: &() = instance.index(0);
        }
    }

    mod index_mut {
        use super::*;

        use core::ops::IndexMut as _;

        #[test]
        fn correct_element() {
            let expected = [0, 1, 2, 3, 4, 5];
            let mut actual = Singly::from_iter(expected);

            for (index, value) in expected.iter().enumerate() {
                assert_eq!(actual.index_mut(index), value);
            }
        }

        #[test]
        fn is_mutable() {
            let mut instance: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

            for index in 0..instance.len() {
                *(instance.index_mut(index)) = 0;
            }

            for element in instance {
                assert_eq!(element, 0);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let mut instance = Singly::<()>::default();

            let _: &() = instance.index_mut(0);
        }
    }

    mod iterator {
        use super::*;

        mod into {
            use super::*;

            #[test]
            fn element_count() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert_eq!(actual.into_iter().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert!(actual.into_iter().eq(expected));
            }

            mod drop {
                use super::*;

                #[test]
                fn drops_unyielded_elements_when_advanced_from_front() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        #[expect(
                            clippy::useless_conversion,
                            reason = "explicitly testing into iterator"
                        )]
                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }))
                            .into_iter();

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_back() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        #[expect(
                            clippy::useless_conversion,
                            reason = "explicitly testing into iterator"
                        )]
                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }))
                            .into_iter();

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next_back());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_both_ends() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for front in 0..ELEMENTS {
                        for back in front..ELEMENTS {
                            let dropped = DropCounter::new_counter();

                            #[expect(
                                clippy::useless_conversion,
                                reason = "explicitly testing into iterator"
                            )]
                            let mut actual =
                                Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                    DropCounter::new(&dropped)
                                }))
                                .into_iter();

                            for _ in 0..front {
                                // Lifetime is passed to caller.
                                drop(actual.next());
                            }

                            for _ in front..back {
                                // Lifetime is passed to caller.
                                drop(actual.next_back());
                            }

                            // The above drops in caller scope, not the
                            // destructor being tested, so reset counter.
                            let expected = ELEMENTS - dropped.replace(0);

                            // Now we drop the iterator, so we expect all
                            // remaining elements to be dropped.
                            drop(actual);

                            assert_eq!(dropped.take(), expected);
                        }
                    }
                }
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.into_iter().rev().len(), expected.len());
                }

                #[test]
                fn in_order() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert!(actual.into_iter().rev().eq(expected.into_iter().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.size_hint(), (expected.len(), Some(expected.len())));
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.len(), expected.len());
                }

                #[test]
                fn updates() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let mut remaining = actual.len();

                    while let Some(_) = actual.next() {
                        remaining -= 1;

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let mut actual = Singly::<()>::default();

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn exhausted() {
                    let mut actual = Singly::from_iter([()]);

                    // Exhaust the elements.
                    let _: () = actual.next().expect("the one element");

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }
            }
        }

        mod from {
            use super::*;

            #[test]
            fn empty() {
                let actual: Singly<()> = core::iter::empty().collect();

                assert!(actual.elements.is_none());
            }

            #[test]
            fn initializes_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert!(actual.eq(expected));
            }

            #[test]
            fn handles_oversized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (usize::MAX, Some(usize::MAX)),
                }
                .collect();

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_undersized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (0, Some(0)),
                }
                .collect();

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_invalid_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (usize::MAX, Some(0)),
                }
                .collect();

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_unbounded_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (0, None),
                }
                .collect();

                assert_eq!(actual.len(), expected.len());
            }
        }

        mod extend {
            use super::*;

            #[test]
            fn appends_elements() {
                let preexisting = [0, 1, 2];
                let mut actual: Singly<_> = preexisting.into_iter().collect();

                let expected = [3, 4, 5];
                actual.extend(expected.iter().copied());

                #[expect(clippy::shadow_unrelated, reason = "elements from them")]
                for (actual, expected) in actual.skip(preexisting.len()).zip(expected) {
                    assert_eq!(actual, expected);
                }
            }

            #[test]
            fn does_not_modify_other_elements() {
                let expected = [0, 1, 2];

                let mut actual: Singly<_> = expected.into_iter().collect();

                actual.extend([3, 4, 5]);

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn into_empty_instance() {
                let mut actual = Singly::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];

                actual.extend(expected.iter().copied());

                assert!(actual.eq(expected));
            }

            #[test]
            fn from_empty_iterator() {
                let mut actual = Singly::<()>::default();

                actual.extend(core::iter::empty());

                assert!(actual.elements.is_none());
            }

            #[test]
            fn handles_oversized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual = Singly::default();

                actual.extend(SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (usize::MAX, Some(usize::MAX)),
                });

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_undersized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual = Singly::default();

                actual.extend(SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (0, Some(0)),
                });

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_invalid_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual = Singly::default();

                actual.extend(SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (usize::MAX, Some(0)),
                });

                assert_eq!(actual.len(), expected.len());
            }

            #[test]
            fn handles_unbounded_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual = Singly::default();

                actual.extend(SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (0, None),
                });

                assert_eq!(actual.len(), expected.len());
            }
        }
    }

    mod collection {
        use super::*;

        mod count {
            use super::*;

            #[test]
            fn number_of_elements() {
                let actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                assert_eq!(Collection::count(&actual), 6);
            }
        }
    }

    mod linear {
        use super::*;

        mod iter {
            use super::*;

            #[test]
            fn element_count() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert_eq!(actual.iter().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                assert!(actual.iter().eq(expected.iter()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert!(actual.iter().rev().eq(expected.iter().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(
                        actual.iter().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let actual: Singly<_> = [0, 1, 2, 3, 4, 5].iter().copied().collect();

                    let mut actual = actual.iter();

                    let mut remaining = actual.len();

                    while let Some(_) = actual.next() {
                        remaining -= 1;

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let actual = Singly::<()>::default();

                    let mut actual = actual.iter();

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn exhausted() {
                    let actual: Singly<_> = [()].into_iter().collect();

                    let mut actual = actual.iter();

                    // Exhaust the elements.
                    let _: &() = actual.next().expect("the one element");

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }
            }
        }

        mod iter_mut {
            use super::*;

            #[test]
            fn element_count() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                assert_eq!(actual.iter_mut().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                assert!(actual.iter_mut().eq(expected.iter_mut()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter_mut().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert!(actual.iter_mut().rev().eq(expected.iter_mut().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(
                        actual.iter_mut().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter_mut().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    let mut actual = actual.iter_mut();

                    let mut remaining = actual.len();

                    while let Some(_) = actual.next() {
                        remaining -= 1;

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let mut actual = Singly::<()>::default();

                    let mut actual = actual.iter_mut();

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn exhausted() {
                    let mut actual: Singly<_> = [()].into_iter().collect();

                    let mut actual = actual.iter_mut();

                    // Exhaust the elements.
                    let _: &() = actual.next().expect("the one element");

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }
            }
        }

        mod at {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Singly<_> = expected.iter().copied().collect();

                for (index, element) in expected.iter().enumerate() {
                    assert_eq!(actual.at(index), Some(element));
                }
            }

            #[test]
            fn none_when_index_out_of_bounds() {
                let actual = Singly::<()>::default();

                assert!(actual.at(0).is_none());
            }
        }

        mod at_mut {
            use super::*;

            #[test]
            fn correct_element() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for (index, element) in expected.iter_mut().enumerate() {
                    assert_eq!(actual.at_mut(index), Some(element));
                }
            }

            #[test]
            fn is_mutable() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                for index in 0..actual.len() {
                    let element = actual.at_mut(index).expect("within bounds");

                    *element = 12345;
                }

                for element in actual {
                    assert_eq!(element, 12345);
                }
            }

            #[test]
            fn none_when_index_out_of_bounds() {
                let mut actual = Singly::<()>::default();

                assert!(actual.at_mut(0).is_none());
            }
        }

        mod first {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.first(), Some(&element));

                    _ = actual.next();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Singly::<()>::default();

                assert_eq!(actual.first(), None);
            }
        }

        mod last {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected.into_iter().rev() {
                    assert_eq!(Linear::last(&actual), Some(&element));

                    _ = actual.next_back();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Singly::<()>::default();

                assert_eq!(Linear::last(&actual), None);
            }
        }

        mod first_mut {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for mut element in expected {
                    assert_eq!(actual.first_mut(), Some(&mut element));

                    _ = actual.next();
                }
            }

            #[test]
            fn is_mutable() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                let element = actual.first_mut().expect("the first element");

                *element = 12345;

                assert_eq!(actual.next(), Some(12345));
            }

            #[test]
            fn does_not_modify_other_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.first_mut().expect("the first element");

                assert!(actual.eq(expected));
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.first_mut(), None);
            }
        }

        mod last_mut {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for mut element in expected.into_iter().rev() {
                    assert_eq!(actual.last_mut(), Some(&mut element));

                    _ = actual.next_back();
                }
            }

            #[test]
            fn is_mutable() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                let element = actual.last_mut().expect("the first element");

                *element = 12345;

                assert_eq!(actual.next_back(), Some(12345));
            }

            #[test]
            fn does_not_modify_other_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.last_mut().expect("the first element");

                assert!(actual.eq(expected));
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.last_mut(), None);
            }
        }
    }

    mod list {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual[2], 12345);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.insert(2, 12345).expect("successful allocation");

                *actual = 54321;

                assert_eq!(actual, &mut 54321);
            }

            #[test]
            fn does_not_modify_leading_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.insert(INDEX, 12345).expect("successful allocation");

                for index in 0..INDEX {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.insert(INDEX, 12345).expect("successful allocation");

                for index in INDEX..expected.len() {
                    assert_eq!(actual[index + 1], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Singly::<usize>::default();

                _ = actual.insert(0, 12345).expect("successful allocation");
            }

            #[test]
            fn can_prepend() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.insert(0, 12345).expect("successful allocation");

                assert_eq!(actual[0], 12345);
            }

            #[test]
            fn can_append() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.insert(6, 12345).expect("successful allocation");

                assert_eq!(actual[6], 12345);
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.remove(0).expect("valid index");

                assert_eq!(actual.len(), expected.len() - 1);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    let removed = actual.remove(0).expect("front element");

                    assert_eq!(removed, element);
                }
            }

            #[test]
            fn does_not_modify_leading_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.remove(INDEX).expect("valid index");

                for index in 0..INDEX {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.remove(INDEX).expect("valid index");

                for index in INDEX..expected.len() - 1 {
                    assert_eq!(actual[index], expected[index + 1]);
                }
            }

            #[test]
            fn none_when_index_out_of_bounds() {
                let mut actual = Singly::<()>::default();

                assert!(actual.remove(0).is_none());
            }
        }

        mod prepend {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.prepend(0).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Singly<_> = [1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.prepend(0).expect("successful allocation");

                assert_eq!(actual[0], 0);
            }

            #[test]
            fn yields_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.prepend(0).expect("successful allocation");

                assert_eq!(actual, &mut 0);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.prepend(0).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.prepend(0).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index + 1], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Singly::<usize>::default();

                _ = actual.prepend(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod append {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.append(5).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4].into_iter().collect();

                _ = actual.append(5).expect("successful allocation");

                assert_eq!(actual[5], 5);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.append(5).expect("successful allocation");

                assert_eq!(actual, &mut 5);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.append(5).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_leading_elements() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.append(5).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Singly::<usize>::default();

                _ = actual.append(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod front {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.front();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for offset in 1..=expected.len() {
                    _ = actual.front();

                    assert!(actual.iter().eq(expected[offset..].iter()));
                }

                assert!(actual.elements.is_none());
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.front(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.front(), None);
            }
        }

        mod back {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.back();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_leading_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for offset in (0..expected.len()).rev() {
                    _ = actual.back();

                    assert!(actual.iter().eq(expected[..offset].iter()));
                }

                assert!(actual.elements.is_none());
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected.into_iter().rev() {
                    assert_eq!(actual.back(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.back(), None);
            }
        }

        mod drain {
            use super::*;

            mod iterator {
                use super::*;

                #[test]
                fn yields_no_elements_when_empty() {
                    let mut actual = Singly::<()>::default();

                    let mut actual = actual.drain(..);

                    assert_eq!(actual.next(), None);

                    drop(actual);
                }

                #[test]
                fn yields_no_elements_when_start_of_range_is_out_of_bounds() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let mut actual = actual.drain(6..);

                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn yields_elements_when_end_of_range_is_out_of_bounds() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = actual.drain(..usize::MAX);

                    assert!(actual.eq([0, 1, 2, 3, 4, 5]));
                }

                #[test]
                fn element_count() {
                    let mut expected = vec![0, 1, 2, 3, 4, 5];
                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert_eq!(actual.drain(1..4).count(), expected.drain(1..4).count());
                }

                #[test]
                fn in_order() {
                    let mut expected = vec![0, 1, 2, 3, 4, 5];
                    let mut actual: Singly<_> = expected.iter().copied().collect();

                    assert!(actual.drain(1..4).eq(expected.drain(1..4)));
                }

                mod double_ended {
                    use super::*;

                    #[test]
                    fn yields_no_elements_when_empty() {
                        let mut actual = Singly::<()>::default();

                        let mut actual = actual.drain(..);

                        assert_eq!(actual.next_back(), None);

                        drop(actual);
                    }

                    #[test]
                    fn yields_no_elements_when_start_of_range_is_out_of_bounds() {
                        let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                        let mut actual = actual.drain(6..);

                        assert_eq!(actual.next_back(), None);
                    }

                    #[test]
                    fn yields_elements_when_end_of_range_is_out_of_bounds() {
                        let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = actual.drain(..usize::MAX).rev();

                        assert!(actual.eq([5, 4, 3, 2, 1, 0]));
                    }

                    #[test]
                    fn element_count() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        assert_eq!(
                            actual.drain(1..4).rev().count(),
                            expected.drain(1..4).rev().count()
                        );
                    }

                    #[test]
                    fn in_order() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        assert!(actual.drain(1..4).rev().eq(expected.drain(1..4).rev()));
                    }

                    #[test]
                    fn prevents_elements_from_being_yielded_more_than_once() {
                        let mut underlying = Singly::from_iter([0, 1, 2, 0]);

                        let mut actual = underlying.drain(1..=2);

                        // make head and tail meet.
                        _ = actual.next().expect("the element with value '1'");
                        _ = actual.next_back().expect("the element with value '2'");

                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }
                }

                mod exact_size {
                    use super::*;

                    #[test]
                    fn hint() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        let expected = expected.drain(1..4);

                        assert_eq!(
                            actual.drain(1..4).size_hint(),
                            (expected.len(), Some(expected.len()))
                        );
                    }

                    #[test]
                    fn len() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        assert_eq!(actual.drain(1..4).len(), expected.drain(1..4).len());
                    }
                }

                mod fused {
                    use super::*;

                    #[test]
                    fn when_empty() {
                        let mut actual = Singly::<()>::default();
                        let mut actual = actual.drain(..);

                        // Yields `None` at least once.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);

                        // Continues to yield `None`.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }

                    #[test]
                    fn when_exhausted() {
                        let mut actual: Singly<_> = [()].into_iter().collect();
                        let mut actual = actual.drain(..);

                        // Exhaust the elements.
                        let _: () = actual.next().expect("the one element");

                        // Yields `None` at least once.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);

                        // Continues to yield `None`.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }
                }
            }

            mod drop {
                use super::*;

                #[test]
                fn drops_unyielded_elements_when_advanced_from_front() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }));

                        let mut actual = actual.drain(..);

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_back() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }));

                        let mut actual = actual.drain(..);

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next_back());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_both_ends() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for front in 0..ELEMENTS {
                        for back in front..ELEMENTS {
                            let dropped = DropCounter::new_counter();

                            let mut actual =
                                Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                    DropCounter::new(&dropped)
                                }));

                            let mut actual = actual.drain(..);

                            for _ in 0..front {
                                // Lifetime is passed to caller.
                                drop(actual.next());
                            }

                            for _ in front..back {
                                // Lifetime is passed to caller.
                                drop(actual.next_back());
                            }

                            // The above drops in caller scope, not the
                            // destructor being tested, so reset counter.
                            let expected = ELEMENTS - dropped.replace(0);

                            // Now we drop the iterator, so we expect all
                            // remaining elements to be dropped.
                            drop(actual);

                            assert_eq!(dropped.take(), expected);
                        }
                    }
                }

                #[test]
                fn can_drain_all_elements() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.drain(..));

                    assert!(actual.elements.is_none());
                }

                #[test]
                fn does_not_modify_leading_elements() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 0..ELEMENTS {
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        drop(actual.drain(start..));

                        assert!(actual.iter().eq(expected[..start].iter()));
                    }
                }

                #[test]
                fn does_not_modify_trailing_elements() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for end in 0..ELEMENTS {
                        let mut actual: Singly<_> = expected.iter().copied().collect();

                        drop(actual.drain(..end));

                        assert!(actual.iter().eq(expected[end..].iter()));
                    }
                }

                #[test]
                fn combines_leading_and_trailing_elements() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 0..ELEMENTS {
                        for end in start..ELEMENTS {
                            let mut actual: Singly<_> = expected.iter().copied().collect();

                            drop(actual.drain(start..end));

                            let expected_leading = expected[..start].iter();
                            let expected_trailing = expected[end..].iter();

                            assert!(actual.iter().eq(expected_leading.chain(expected_trailing)));
                        }
                    }
                }
            }
        }

        mod withdraw {
            use super::*;

            mod iterator {
                use super::*;

                #[test]
                fn element_count() {
                    let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = underlying.withdraw(|element| element % 2 == 0);

                    assert_eq!(actual.count(), 3);
                }

                #[test]
                fn in_order() {
                    let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = underlying.withdraw(|element| element % 2 == 0);

                    assert!(actual.eq([0, 2, 4]));
                }

                #[test]
                fn combines_retained_elements() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.withdraw(|element| element == &1));

                    assert!(actual.eq([0, 2, 3, 4, 5]));
                }

                #[test]
                fn size_hint() {
                    let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    let mut actual = underlying.withdraw(|element| element % 2 == 0);

                    assert_eq!(actual.size_hint(), (0, Some(6)));

                    _ = actual.next().expect("element with value 0");
                    assert_eq!(actual.size_hint(), (0, Some(5)));

                    _ = actual.next().expect("element with value 2");
                    assert_eq!(actual.size_hint(), (0, Some(3)));

                    _ = actual.next().expect("element with value 4");
                    assert_eq!(actual.size_hint(), (0, Some(1)));

                    _ = actual.next();
                    assert_eq!(actual.size_hint(), (0, Some(0)));
                }

                mod double_ended {
                    use super::*;

                    #[test]
                    fn element_count() {
                        let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = underlying.withdraw(|element| element % 2 == 0).rev();

                        assert_eq!(actual.count(), 3);
                    }

                    #[test]
                    fn in_order() {
                        let mut underlying = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = underlying.withdraw(|element| element % 2 == 0).rev();

                        assert!(actual.eq([4, 2, 0]));
                    }

                    #[test]
                    fn combines_retained_elements() {
                        let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                        drop(actual.withdraw(|element| element == &1).rev());

                        assert!(actual.eq([0, 2, 3, 4, 5]));
                    }

                    #[test]
                    fn prevents_elements_from_being_yielded_more_than_once() {
                        let mut underlying = Singly::from_iter([0, 1, 2, 0]);

                        let mut actual = underlying.withdraw(|element| element != &0);

                        // make head and tail meet.
                        _ = actual.next().expect("the element with value '1'");
                        _ = actual.next_back().expect("the element with value '2'");

                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }
                }

                mod fused {
                    use super::*;

                    #[test]
                    fn empty() {
                        let mut underlying = Singly::from_iter([0]);
                        let mut actual = underlying.withdraw(|element| element % 2 == 0);

                        // Exhaust the elements.
                        _ = actual.next().expect("the one element");

                        // Yields `None` at least once.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);

                        // Continues to yield `None`.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }

                    #[test]
                    fn exhausted() {
                        let mut underlying = Singly::<usize>::default();
                        let mut actual = underlying.withdraw(|element| element % 2 == 0);

                        // Yields `None` at least once.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);

                        // Continues to yield `None`.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }
                }
            }

            mod drop {
                use super::*;

                #[test]
                fn drops_unyielded_elements_when_advanced_from_front() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }));

                        let mut actual = actual.withdraw(|_| true);

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_back() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for yielded in 0..ELEMENTS {
                        let dropped = DropCounter::new_counter();

                        let mut actual =
                            Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                DropCounter::new(&dropped)
                            }));

                        let mut actual = actual.withdraw(|_| true);

                        for _ in 0..yielded {
                            // Lifetime is passed to caller.
                            drop(actual.next_back());
                        }

                        // The above drops in caller scope, not the
                        // destructor being tested, so reset counter.
                        debug_assert_eq!(dropped.replace(0), yielded);

                        // Now we drop the iterator, so we expect all
                        // remaining elements to be dropped.
                        drop(actual);

                        assert_eq!(dropped.take(), ELEMENTS - yielded);
                    }
                }

                #[test]
                fn drops_unyielded_elements_when_advanced_from_both_ends() {
                    use crate::test::mock::DropCounter;

                    const ELEMENTS: usize = 8;

                    for front in 0..ELEMENTS {
                        for back in front..ELEMENTS {
                            let dropped = DropCounter::new_counter();

                            let mut actual =
                                Singly::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
                                    DropCounter::new(&dropped)
                                }));

                            let mut actual = actual.withdraw(|_| true);

                            for _ in 0..front {
                                // Lifetime is passed to caller.
                                drop(actual.next());
                            }

                            for _ in front..back {
                                // Lifetime is passed to caller.
                                drop(actual.next_back());
                            }

                            // The above drops in caller scope, not the
                            // destructor being tested, so reset counter.
                            let expected = ELEMENTS - dropped.replace(0);

                            // Now we drop the iterator, so we expect all
                            // remaining elements to be dropped.
                            drop(actual);

                            assert_eq!(dropped.take(), expected);
                        }
                    }
                }

                #[test]
                fn can_withdraw_all_elements() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.withdraw(|_| true));

                    assert!(actual.elements.is_none());
                }

                #[test]
                fn does_not_modify_retained_elements() {
                    let mut actual = Singly::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.withdraw(|element| element % 2 == 0));

                    assert!(actual.eq([1, 3, 5]));
                }
            }
        }
    }

    mod stack {
        use super::*;

        use super::super::super::super::Stack as _;

        mod push {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Singly<_> = [1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual[0], 0);
            }

            #[test]
            fn yields_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.push(0).expect("successful allocation");

                assert_eq!(actual, &mut 0);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.push(0).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index + 1], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Singly::<usize>::default();

                _ = actual.push(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod pop {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.pop();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for offset in 1..=expected.len() {
                    _ = actual.pop();

                    assert!(actual.iter().eq(expected[offset..].iter()));
                }

                assert!(actual.elements.is_none());
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.pop(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.pop(), None);
            }
        }

        mod peek {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.peek(), Some(&element));

                    _ = actual.pop();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Singly::<()>::default();

                assert_eq!(actual.peek(), None);
            }
        }
    }

    mod queue {
        use super::*;

        use super::super::super::super::Queue as _;

        mod push {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Singly<_> = [0, 1, 2, 3, 4].into_iter().collect();

                _ = actual.push(5).expect("successful allocation");

                assert_eq!(actual[5], 5);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.push(5).expect("successful allocation");

                assert_eq!(actual, &mut 5);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                let actual = actual.push(5).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_leading_elements() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                _ = actual.push(5).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Singly::<usize>::default();

                _ = actual.push(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod pop {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.pop();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for offset in 1..=expected.len() {
                    _ = actual.pop();

                    assert!(actual.iter().eq(expected[offset..].iter()));
                }

                assert!(actual.elements.is_none());
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.pop(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Singly::<()>::default();

                assert_eq!(actual.pop(), None);
            }
        }

        mod peek {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Singly<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.peek(), Some(&element));

                    _ = actual.pop();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Singly::<()>::default();

                assert_eq!(actual.peek(), None);
            }
        }
    }
}

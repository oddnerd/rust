//! Implementation of [`Dope`].

use super::Array;
use super::Collection;
use super::Linear;

use core::ptr::NonNull;

/// Lightweight access to a contigious buffer of memory.
///
/// A [Dope Vector](https://en.wikipedia.org/wiki/Dope_vector) comprises of a
/// pointer and length pair leveraging compile-time type information alongside
/// pointer arithmetic to distinguish between individual elements.
///
/// This is equivalent to Rust's slice ([`[T]`]([`slice`])) or C++'s span
/// ([`std::span`][span]) and views ([`std::string_view`][string_view]).
///
/// Note that is is strictly a mutable variant, hence you cannot safely
/// construct an instance from memory you only have an immutable reference to,
/// even if you promise to not call mutable methods. The expected use case is
/// constructing from _owned_ memory.
///
/// [span]: https://en.cppreference.com/w/cpp/container/span
/// [string_view]: https://en.cppreference.com/w/cpp/string/basic_string_view
#[derive(Clone, Copy)]
pub struct Dope<'a, T> {
    /// Pointer to the start of the array.
    ptr: NonNull<T>,

    /// Number of elements within the array.
    count: usize,

    /// Bind lifetime to underlying memory buffer.
    lifetime: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T: 'a> Dope<'a, T> {
    /// Construct from a pointer to an array and the number of elements.
    ///
    /// # Safety
    /// * `ptr` must have an address aligned for access to `T`.
    /// * `ptr` must point to one contigious allocated object.
    /// * `ptr` must point to `len` consecutive initialized instances of `T`.
    /// * Cannot use this object to modify immutable underlying memory.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let ptr = expected.as_mut_ptr();
    /// let ptr = unsafe { core::ptr::NonNull::new_unchecked(ptr) };
    /// let actual = unsafe { Dope::new(ptr, expected.len()) };
    ///
    /// assert!(actual.iter().eq(expected.iter()));
    /// ```
    #[must_use]
    pub unsafe fn new(ptr: NonNull<T>, count: usize) -> Self {
        Self {
            ptr,
            count,
            lifetime: core::marker::PhantomData,
        }
    }
}

impl<'a, T: 'a> From<&'a mut [T]> for Dope<'a, T> {
    /// Construct from an existing mutable [`slice`].
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let mut clone = expected.clone();
    ///
    /// let actual = Dope::from(clone.as_mut_slice());
    ///
    /// assert!(actual.iter().eq(expected.iter()));
    /// ```
    fn from(slice: &'a mut [T]) -> Self {
        Self {
            ptr: {
                let ptr = slice.as_mut_ptr();

                // SAFETY: `slice` exists => pointer is non-null.
                unsafe { NonNull::new_unchecked(ptr) }
            },
            count: slice.len(),
            lifetime: core::marker::PhantomData,
        }
    }
}

impl<'a, T: 'a> core::ops::Index<usize> for Dope<'a, T> {
    type Output = T;

    /// Query the element `index` positions from the start.
    ///
    /// # Panics
    /// This method has the precondition that the `index` is within bounds.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual = Dope::from(expected.as_mut_slice());
    ///
    /// for index in 0..=5{
    ///     use core::ops::Index;
    ///     assert_eq!(actual.index(index), &index);
    /// }
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.count, "index out of bounds");

        let ptr = {
            let ptr = self.ptr.as_ptr();

            // SAFETY: `index` in bounds => aligned within allocated object.
            unsafe { ptr.add(index) }
        };

        // SAFETY:
        // * `index` in bounds => pointed to `T` is initialized.
        // * lifetime bound to input object => valid lifetime to return.
        unsafe { &*ptr }
    }
}

impl<'a, T: 'a> core::ops::IndexMut<usize> for Dope<'a, T> {
    /// Obtain a reference to the element `index` positions from the start.
    ///
    /// # Panics
    /// This method has the precondition that the `index` is within bounds.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let mut actual = Dope::from(expected.as_mut_slice());
    ///
    /// for mut index in 0..=5 {
    ///     use core::ops::IndexMut;
    ///     assert_eq!(actual.index_mut(index), &mut index);
    /// }
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.count, "index out of bounds");

        let ptr = {
            let ptr = self.ptr.as_ptr();

            // SAFETY: `index` in bounds => aligned within allocated object.
            unsafe { ptr.add(index) }
        };

        // SAFETY:
        // * `index` in bounds => pointed to `T` is initialized.
        // * lifetime bound to input object => valid lifetime to return.
        unsafe { &mut *ptr }
    }
}

impl<'a, T: 'a + PartialEq> PartialEq for Dope<'a, T> {
    /// Query if the elements referenced to/contained are the same as `other`.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut left = [0, 1, 2, 3, 4, 5];
    /// let mut right = left.clone();
    ///
    /// let left = unsafe { Dope::from(left.as_mut_slice()) };
    /// let right = unsafe { Dope::from(right.as_mut_slice()) };
    ///
    /// assert_eq!(left, right);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'a, T: 'a + Eq> Eq for Dope<'a, T> {}

impl<'a, T: 'a + core::fmt::Debug> core::fmt::Debug for Dope<'a, T> {
    /// List the elements referenced to/contained.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual = Dope::from(expected.as_mut_slice());
    ///
    /// assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T: 'a> Collection for Dope<'a, T> {
    type Element = T;

    /// Query how many elements are referenced to/contained.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::Collection;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual = Dope::from(expected.as_mut_slice());
    ///
    /// assert_eq!(Collection::count(&actual), expected.len());
    /// ```
    fn count(&self) -> usize {
        self.count
    }
}

impl<'a, T: 'a> Linear for Dope<'a, T> {
    /// Immutably iterate the elements in order.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual = Dope::from(expected.as_mut_slice());
    ///
    /// for (index, element) in actual.iter().enumerate() {
    ///     assert_eq!(element, &index);
    /// }
    /// ```
    fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Self::Element> + ExactSizeIterator + core::iter::FusedIterator
    {
        // SAFETY:
        // * Pointer is aligned.
        // * Points to one allocated object.
        // * Points to `count` contigious initialized instance of `T`.
        unsafe { super::Iter::new(self.ptr, self.count) }
    }

    /// Mutably iterate the elements in order.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let mut actual = Dope::from(expected.as_mut_slice());
    ///
    /// for (mut index, element) in actual.iter_mut().enumerate() {
    ///     assert_eq!(element, &mut index);
    /// }
    /// ```
    fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut Self::Element>
    + ExactSizeIterator
    + core::iter::FusedIterator {
        // SAFETY:
        // * Pointer is aligned.
        // * Points to one allocated object.
        // * Points to `count` contigious initialized instance of `T`.
        unsafe { super::IterMut::new(self.ptr, self.count) }
    }
}

impl<'a, T: 'a> Array for Dope<'a, T> {
    /// Obtain an immutable pointer to the underlying contigious memory buffer.
    ///
    /// # Safety
    /// * `self` must outlive the resultant pointer.
    /// * Cannot write to resultant pointer or any pointer derived from it.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual = Dope::from(expected.as_mut_slice());
    ///
    /// assert_eq!(actual.as_ptr(), expected.as_ptr());
    /// ```
    fn as_ptr(&self) -> *const Self::Element {
        self.ptr.as_ptr().cast_const()
    }

    /// Obtain an immutable pointer to the underlying contigious memory buffer.
    ///
    /// # Safety
    /// * `self` must outlive the resultant pointer.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dope;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let mut actual = Dope::from(expected.as_mut_slice());
    ///
    /// assert_eq!(actual.as_mut_ptr(), expected.as_mut_ptr());
    /// ```
    fn as_mut_ptr(&mut self) -> *mut Self::Element {
        self.ptr.as_ptr()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod method {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn correct_size() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert_eq!(actual.count, expected.len());
            }

            #[test]
            fn correct_pointer() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert_eq!(actual.ptr.as_ptr(), expected.as_mut_ptr());
            }
        }
    }

    mod from {
        use super::*;

        mod primitive_slice {
            use super::*;

            #[test]
            fn correct_size() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.count, expected.len());
            }

            #[test]
            fn correct_pointer() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.ptr.as_ptr(), expected.as_mut_ptr());
            }
        }
    }

    mod index {
        use super::*;

        use core::ops::Index as _;

        #[test]
        fn correct_element() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let actual = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            for (index, value) in expected.iter().enumerate() {
                assert_eq!(actual.index(index), value);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let mut underlying: [(); 0] = [];
            let instance = Dope::from(underlying.as_mut_slice());

            let _: &() = instance.index(0);
        }
    }

    mod index_mut {
        use super::*;

        use core::ops::IndexMut as _;

        #[test]
        fn correct_element() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let mut actual = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            for (index, value) in expected.iter_mut().enumerate() {
                assert_eq!(actual.index_mut(index), value);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let mut underlying: [(); 0] = [];
            let mut instance = Dope::from(underlying.as_mut_slice());

            let _: &() = instance.index_mut(0);
        }

        #[test]
        fn is_mutable() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let mut actual = Dope::from(expected.as_mut_slice());

            for index in 0..actual.count() {
                *actual.index_mut(index) = 0;
            }

            for element in expected {
                assert_eq!(element, 0);
            }
        }
    }

    mod copy {
        use super::*;

        #[test]
        fn has_elements() {
            let mut underlying = [0, 1, 2, 3, 4, 5];
            let expected = Dope::from(underlying.as_mut_slice());

            let actual = expected;

            assert_eq!(actual.count, expected.count);
        }

        #[test]
        fn is_equivalent() {
            let mut underlying = [0, 1, 2, 3, 4, 5];
            let expected = Dope::from(underlying.as_mut_slice());

            let actual = expected;

            assert_eq!(actual, expected);
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn eq_when_same_elements() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let first = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            let second = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            assert_eq!(first, second);
        }

        #[test]
        fn ne_when_different_elements() {
            let mut zero = [0];
            let zero = {
                let ptr = zero.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, zero.len()) }
            };

            let mut one = [1];
            let one = {
                let ptr = one.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, one.len()) }
            };

            assert_ne!(zero, one);
        }

        #[test]
        fn is_symmetric() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let first = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            let second = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            // `first == second` <=> `second == first`
            assert_eq!(first, second);
            assert_eq!(second, first);
        }

        #[test]
        fn is_transitive() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let first = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            let second = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            let third = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            // `first == second && second == third` => `first == third`
            assert_eq!(first, second);
            assert_eq!(second, third);
            assert_eq!(third, first);
        }

        #[test]
        fn is_reflexive() {
            let mut expected = [0, 1, 2, 3, 4, 5];

            let actual = {
                let ptr = expected.as_mut_ptr();
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                unsafe { Dope::new(ptr, expected.len()) }
            };

            assert_eq!(actual, actual);
        }
    }

    mod fmt {
        use super::*;

        mod debug {
            use super::*;

            #[test]
            fn is_elements() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
            }
        }
    }

    mod collection {
        use super::*;

        mod count {
            use super::*;

            #[test]
            fn initialized_elements() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert_eq!(Collection::count(&actual), expected.len());
            }

            #[test]
            fn zero_when_empty() {
                let mut expected: [(); 0] = [];
                let actual = Dope::from(expected.as_mut_slice());

                assert_eq!(Collection::count(&actual), 0);
            }
        }
    }

    mod linear {
        use super::*;

        mod iter {
            use super::*;

            #[test]
            fn element_count() {
                let mut expected = [0, 1, 2, 3, 4, 5];
                let actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.iter().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert!(actual.iter().eq(expected.iter()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let mut expected = [0, 1, 2, 3, 4, 5];
                    let actual = Dope::from(expected.as_mut_slice());

                    assert_eq!(actual.iter().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert!(actual.iter().rev().eq(expected.iter().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert_eq!(
                        actual.iter().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert_eq!(actual.iter().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    let mut actual = actual.iter();

                    for remaining in (0..expected.len()).rev() {
                        _ = actual.next();

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let mut expected: [(); 0] = [];
                    let actual = Dope::from(expected.as_mut_slice());
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
                    let mut expected = [()];
                    let actual = Dope::from(expected.as_mut_slice());
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
                let mut expected = [0, 1, 2, 3, 4, 5];
                let mut actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.iter_mut().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let mut expected = [0, 1, 2, 3, 4, 5];

                let mut actual = {
                    let ptr = expected.as_mut_ptr();
                    let ptr = unsafe { NonNull::new_unchecked(ptr) };
                    unsafe { Dope::new(ptr, expected.len()) }
                };

                assert!(actual.iter_mut().eq(expected.iter_mut()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let mut expected = [0, 1, 2, 3, 4, 5];
                    let mut actual = Dope::from(expected.as_mut_slice());

                    assert_eq!(actual.iter_mut().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let mut actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert!(actual.iter_mut().rev().eq(expected.iter_mut().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let mut actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert_eq!(
                        actual.iter_mut().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let mut actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    assert_eq!(actual.iter_mut().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let mut expected = [0, 1, 2, 3, 4, 5];

                    let mut actual = {
                        let ptr = expected.as_mut_ptr();
                        let ptr = unsafe { NonNull::new_unchecked(ptr) };
                        unsafe { Dope::new(ptr, expected.len()) }
                    };

                    let mut actual = actual.iter_mut();

                    for remaining in (0..expected.len()).rev() {
                        _ = actual.next();

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let mut expected: [(); 0] = [];
                    let mut actual = Dope::from(expected.as_mut_slice());
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
                    let mut expected = [()];
                    let mut actual = Dope::from(expected.as_mut_slice());
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
    }

    mod array {
        use super::*;

        mod as_ptr {
            use super::*;

            #[test]
            fn correct_address() {
                let mut expected = [0, 1, 2, 3, 4, 5];
                let actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.as_ptr(), expected.as_ptr());
            }
        }

        mod as_mut_ptr {
            use super::*;

            #[test]
            fn correct_address() {
                let mut expected = [0, 1, 2, 3, 4, 5];
                let mut actual = Dope::from(expected.as_mut_slice());

                assert_eq!(actual.as_mut_ptr(), expected.as_mut_ptr());
            }
        }
    }
}

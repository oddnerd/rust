//! Implementation of [`IterMut`].

/// Mutable reference [`Iterator`] over an [`super::super::Array`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IterMut<'a, T: 'a> {
    /// Pointer to the hypothetical next element.
    ptr: std::ptr::NonNull<T>,

    /// Number of elements yet to be yielded.
    count: usize,

    /// Constrain to lifetime of the underlying object.
    lifetime: std::marker::PhantomData<&'a T>,
}

impl<'a, T: 'a> IterMut<'a, T> {
    /// Construct from a pointer to an array and the number of elements.
    ///
    /// # Safety
    /// * `ptr` must have an address aligned for access to `T`.
    /// * `ptr` must point to one contigious allocated object.
    /// * `ptr` must point to `count` consecutive initialized instances of `T`.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::iter::IterMut;
    ///
    /// let mut underlying = [0, 1, 2, 3, 4, 5];
    /// let ptr = std::ptr::NonNull::new(underlying.as_mut_ptr()).unwrap();
    /// let iter = unsafe { IterMut::new(ptr, underlying.len()) };
    ///
    /// assert!(underlying.iter().eq(iter));
    /// ```
    pub unsafe fn new(ptr: std::ptr::NonNull<T>, count: usize) -> Self {
        Self {
            ptr,
            count,
            lifetime: std::marker::PhantomData,
        }
    }
}

impl<'a, T: 'a> std::iter::Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            // SAFETY:
            // * points to initialized element.
            // * lifetime bound to underlying input.
            let result = unsafe { self.ptr.as_mut() };

            self.ptr = unsafe {
                // SAFETY: either within the allocated object or one byte past.
                let ptr = self.ptr.as_ptr().add(1);

                // SAFETY: `add` maintains the non-null requirement.
                std::ptr::NonNull::new_unchecked(ptr)
            };
            self.count -= 1;

            Some(result)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<'a, T: 'a> std::iter::FusedIterator for IterMut<'a, T> {}

impl<'a, T: 'a> std::iter::ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T: 'a> std::iter::DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;

            Some(unsafe {
                // SAFETY: points to final element within the allocated object.
                let ptr = self.ptr.as_ptr().add(self.count);

                // SAFETY:
                // * points to initialized element.
                // * lifetime bound to underlying input.
                ptr.as_mut().unwrap_unchecked()
            })
        } else {
            None
        }
    }
}

impl<'a, T: 'a + std::fmt::Debug> std::fmt::Debug for IterMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: points to `count` initialized instance of `T`.
        let slice = unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.count) };
        f.debug_list().entries(slice).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size_hint_for_normal_types_is_exact_element_count() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert_eq!(underlying.len(), instance.size_hint().0);
        assert_eq!(underlying.len(), instance.size_hint().1.unwrap());
    }

    #[test]
    fn size_hint_for_zero_size_types_is_constructed_count() {
        let count = 256;

        let instance = {
            let ptr = std::ptr::NonNull::<()>::dangling();
            unsafe { IterMut::new(ptr, count) }
        };

        assert_eq!(count, instance.size_hint().0);
        assert_eq!(count, instance.size_hint().1.unwrap());
    }

    #[test]
    fn len_for_normal_types_is_exact_element_count() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert_eq!(underlying.len(), instance.len());
    }

    #[test]
    fn len_for_zero_size_types_is_constructed_count() {
        let underlying = [(), (), (), (), (), ()];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert_eq!(underlying.len(), instance.len());
    }

    #[test]
    fn next_yields_element_count_for_normal_types() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert_eq!(underlying.len(), instance.count());
    }

    #[test]
    fn next_yields_element_count_for_zero_size_types() {
        let underlying = [(), (), (), (), (), ()];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert_eq!(underlying.len(), instance.count());
    }

    #[test]
    fn next_yields_front_element() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert!(underlying.iter().eq(instance));
    }

    #[test]
    fn next_back_yields_element_count_for_normal_types() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        }
        .rev();

        assert_eq!(underlying.len(), instance.count());
    }

    #[test]
    fn next_back_yields_element_count_for_zero_size_types() {
        let underlying = [(), (), (), (), (), ()];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        }
        .rev();

        assert_eq!(underlying.len(), instance.count());
    }

    #[test]
    fn next_back_yields_back_element() {
        let underlying = [0, 1, 2, 3, 4, 5];
        let instance = {
            let ptr = underlying.as_ptr().cast_mut();
            let ptr = unsafe { std::ptr::NonNull::new_unchecked(ptr) };
            unsafe { IterMut::new(ptr, underlying.len()) }
        };

        assert!(underlying.iter().rev().eq(instance.rev()));
    }
}

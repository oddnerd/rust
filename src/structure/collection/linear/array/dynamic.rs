//! Implementation of a [dynamically sized array](https://en.wikipedia.org/wiki/Dynamic_array).

use super::Array;
use super::Collection;
use super::Linear;

/// An [`Array`] which can store a runtime defined number of elements.
///
/// A contigious memory buffer with sequentially laid out elements at alignment
/// divisions. The buffer is lazily heap-allocated to store some number of
/// elements, referred to as the capacity. Elements are sequentially
/// initialized within the buffer as they are appended reducing the capacity.
/// Once the capacity has been exhausted, the buffer is reallocated to contain
/// previously initialized elements followed by new uninitialized capacity.
pub struct Dynamic<T> {
    /// Underlying buffer storing initialized _and_ uninitialized elements.
    data: std::ptr::NonNull<std::mem::MaybeUninit<T>>,

    /// The number of elements which are currently initialized.
    initialized: usize,

    /// The number of elements which are allocated but currently uninitialized.
    allocated: usize,
}

impl<T> Dynamic<T> {
    /// Construct an empty instance.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let instance: Dynamic<()> = Dynamic::new();
    /// assert_eq!(instance.count(), 0);
    /// assert_eq!(instance.capacity(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            data: std::ptr::NonNull::dangling(),
            initialized: 0,
            allocated: 0,
        }
    }

    /// Construct an instance with an allocated buffer for `count` elements.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let instance: Dynamic<()> = Dynamic::with_capacity(4);
    /// assert_eq!(instance.count(), 0);
    /// assert!(instance.capacity() >= 4);
    /// ```
    pub fn with_capacity(count: usize) -> Option<Self> {
        let mut instance = Self::new();
        if instance.reserve(count) {
            Some(instance)
        } else {
            None
        }
    }

    /// Query how many elements could be inserted without allocation.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance: Dynamic<i32> = Dynamic::with_capacity(2).unwrap();
    /// let old_capacity = instance.capacity();
    /// assert!(old_capacity >= 2);
    /// instance.append(1);
    /// instance.append(2);
    /// assert_eq!(instance.capacity(), old_capacity - 2);
    /// ```
    pub fn capacity(&self) -> usize {
        self.allocated
    }

    /// Attempt to allocate space for `capacity` additional elements.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance: Dynamic<()> = Dynamic::new();
    /// assert_eq!(instance.capacity(), 0);
    ///
    /// instance.reserve(16);
    /// assert!(instance.capacity() >= 16);
    /// ```
    pub fn reserve(&mut self, capacity: usize) -> bool {
        if std::mem::size_of::<T>() == 0 {
            self.allocated = usize::MAX;
            return true;
        }

        if self.allocated > capacity || capacity == 0 {
            return true;
        }

        // growth factor of two (2) so capacity is doubled each reallocation.
        let size = match self.initialized.checked_add(capacity) {
            Some(size) => size,
            None => return false,
        }
        .next_power_of_two();

        let layout = match std::alloc::Layout::array::<T>(size) {
            Ok(layout) => layout,
            Err(_) => return false,
        };

        let old_size = self.initialized + self.allocated;

        let ptr = if old_size == 0 {
            // SAFETY: `layout` has non-zero size. TODO
            unsafe { std::alloc::alloc(layout) }
        } else {
            let new_size = layout.size();
            let layout = match std::alloc::Layout::array::<T>(old_size) {
                Ok(layout) => layout,
                Err(_) => return false,
            };

            // SAFETY: `layout` has non-zero size. TODO
            unsafe { std::alloc::realloc(self.data.cast::<u8>().as_ptr(), layout, new_size) }
        };

        // SAFETY: `std::mem::MaybeUninit<T>` has the same layout at `T`.
        let ptr = ptr.cast::<std::mem::MaybeUninit<T>>();

        self.data = match std::ptr::NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => return false,
        };

        self.allocated = size - self.initialized;

        true
    }

    /// Add an element positioned at the next greatest index not yet used.
    ///
    /// Returns true if the element was added, false otherwise such as if
    /// allocation fails.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic<i32>::new();
    /// instance.append(01234);
    /// instance.append(56789);
    /// assert_eq!(instance.count(), 2);
    /// assert_eq!(instance.first(), 01234);
    /// assert_eq!(instance.last(), 56789);
    /// ```
    pub fn append(&mut self, element: T) -> bool {
        if self.allocated == 0 && !self.reserve(1) {
            return false;
        }

        unsafe {
            // SAFETY: the buffer has been allocated.
            let ptr = self.data.as_ptr();

            // SAFETY: this points to the first uninitialized element.
            let ptr = ptr.add(self.initialized);

            // SAFETY:
            // * `ptr` is non-null.
            // * `ptr` is aligned.
            // * the `MaybeUninit<T>` is initialized even if the `T` isn't.
            (*ptr).write(element);
        };

        self.initialized += 1;

        true
    }
}

impl<'a, T: 'a> super::Collection<'a> for Dynamic<T> {
    type Element = T;

    fn count(&self) -> usize {
        self.initialized
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let instance: Dynamic<()> = Dynamic::new();

        assert_eq!(instance.initialized, 0);
        assert_eq!(instance.allocated, 0);
    }

    #[test]
    fn with_capacity() {
        let instance: Dynamic<()> = Dynamic::with_capacity(4).unwrap();

        assert_eq!(instance.initialized, 0);
        assert_eq!(instance.allocated, 4);
    }

    #[test]
    fn count() {
        let instance: Dynamic<()> = Dynamic::new();

        assert_eq!(instance.count(), 0);
    }

    #[test]
    fn capacity() {
        let instance: Dynamic<()> = Dynamic::new();

        assert_eq!(instance.capacity(), 0);
    }

    #[test]
    fn reserve() {
        let mut instance: Dynamic<()> = Dynamic::new();
        assert_eq!(instance.allocated, 0);

        // reserve does initial allocation.
        instance.reserve(8);
        assert_eq!(instance.allocated, 8);

        // reserve does reallocation.
        instance.reserve(16);
        assert_eq!(instance.allocated, 16);

        // reserve does reallocation to shrink.
        instance.reserve(8);
        assert_eq!(instance.allocated, 8);

        // reserve does not remove initialized elements.
        todo!()
    }

    #[test]
    fn append() {
        let mut instance: Dynamic<i32> = Dynamic::new();
        assert_eq!(instance.count(), 0);

        // append to empty instance.
        instance.append(1);
        assert_eq!(instance.count(), 1);

        // append to instance with one element.
        instance.append(2);
        assert_eq!(instance.count(), 2);

        // append to instance with more than one element.
        instance.append(3);
        assert_eq!(instance.count(), 3);

        todo!("check is last element has correct value");
    }
}

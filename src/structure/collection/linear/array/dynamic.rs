//! Implementation of [`Dynamic`].

use super::super::List;
use super::Array;
use super::Collection;
use super::Linear;

use core::mem::MaybeUninit;
use core::ptr::NonNull;

/// An [`Array`] which can store a runtime defined number of elements.
///
/// This is (mostly) equivalent to Rust's [`Vec`] or C++'s
/// [`std::vector`](https://en.cppreference.com/w/cpp/container/vector).
///
/// Contigious memory (one single allocated object) is heap-allocated with
/// alignment and size to store elements of type `T`, referred to as the
/// buffer. The front of the buffer (potentially) contains uninitialized
/// elements, then all initialized elements in the order they were inserted,
/// and finally the back is (potentially) other uninitialized elements.
///
/// The term 'capacity' refers to pre-allocated memory containing those
/// uninitialized elements into which new elements can be added without
/// altering the allocation. This means [`capacity`](`Self::capacity`)
/// elements can be [`insert`](`Self::insert`) without invalidating pointers to
/// the buffer. Note that pointers to specific elements may no longer point to
/// the that element or might point to an uninitialized element as
/// the pre-existing elements may be moved within the buffer to utilize said
/// capacity. In contrast, consuming end-specific capacity via
/// [`prepend`](`Self::prepend`) or [`append`](`Self::append`) alongside
/// [`capacity_front`](`Self::capacity_front`) or
/// [`capacity_back`](`Self::capacity_back`) _will_ maintain pointers to
/// specific elements.
///
/// Capacity may be manually allocated via
/// [`with_capacity`](`Self::with_capacity`) and
/// [`reserve`](`Self::reserve`), or end-specific
/// [`reserve_front`](`Self::reserve_front`) and
/// [`reserve_back`](`Self::reserve_back`) methods which will reallocate
/// thereby invaliding all pointers. Furthermore, capacity can be deallocated
/// (retaining initialized elements) via [`shrink`](`Self::shrink`),
/// or end-specific [`shrink_front`](`Self::shrink_front`) and
/// [`shrink_back`](`Self::shrink_back`). Shrinking when no elements are
/// initialized will deallocate freeing all memory.
///
/// See also: [Wikipedia](https://en.wikipedia.org/wiki/Dynamic_array).
pub struct Dynamic<T> {
    /// Underlying buffer storing initialized _and_ uninitialized elements.
    buffer: NonNull<MaybeUninit<T>>,

    /// The number of uninitialized elements before the initialized ones.
    front_capacity: usize,

    /// The number of elements which are initialized.
    initialized: usize,

    /// The number of uninitialized elements after the initialized ones.
    back_capacity: usize,
}

impl<T> Dynamic<T> {
    /// Attempt to allocate enough memory to store exactly `count` elements.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory allocation fails.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// if let Ok(instance) = Dynamic::<i32>::with_capacity(256) {
    ///     assert_eq!(instance.len(), 0);
    ///     assert_eq!(instance.capacity(), 256);
    ///     assert_eq!(instance.capacity_front(), 256);
    ///     assert_eq!(instance.capacity_back(), 256);
    /// } else {
    ///     panic!("allocation failed");
    /// }
    /// ```
    pub fn with_capacity(count: usize) -> Result<Self, FailedAllocation> {
        let mut instance = Dynamic::<T>::default();

        match instance.reserve_back(count) {
            Ok(_) => Ok(instance),
            Err(_) => Err(FailedAllocation),
        }
    }

    /// Query how many elements could be added without reallocation.
    ///
    /// Note that adding this many elements might still require rearranging the
    /// underlying buffer in non-constant time. This means that although
    /// pointers to the buffer remain valid, they may not point to an
    /// initialized element let alone the element they were assigned to.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<i32>::with_capacity(6).expect("successful allocation");
    ///
    /// // Won't double count capacity.
    /// assert_eq!(instance.capacity(), 6);
    /// assert_eq!(instance.capacity_front(), 6);
    /// assert_eq!(instance.capacity_back(), 6);
    ///
    /// // Reflects when capacity is exhausted.
    /// instance.extend([0, 1, 2, 3, 4, 5]);
    /// assert_eq!(instance.capacity(), 0);
    ///
    /// // Will count any end specific capacity.
    /// instance.reserve_back(256).expect("successful allocation");
    /// assert_eq!(instance.capacity(), 256);
    ///
    /// // Will include both ends' capacity.
    /// instance.reserve_front(256).expect("successful allocation");
    /// assert_eq!(instance.capacity(), 512);
    /// ```
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.front_capacity
            .checked_add(self.back_capacity)
            .map_or_else(
                || unreachable!("allocated more than `isize::MAX` bytes"),
                |capacity| capacity,
            )
    }

    /// How many elements can [`Self::prepend`] in without reallocation.
    ///
    /// This many end-specific insertions will be constant time without
    /// possibility of error. Moreover, this maintains pointer validity
    /// even to specific elements.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// // Constructing with generic capacity.
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    /// assert_eq!(instance.capacity_front(), 256);
    ///
    /// // Reserving for specific end of the buffer.
    /// instance.reserve_front(512).expect("successful allocation");
    /// assert_eq!(instance.capacity_front(), 512);
    ///
    /// // Reserving for opposite end of the buffer, but be empty.
    /// instance.reserve_back(1024).expect("successful allocation");
    /// assert_eq!(instance.capacity_front(), 1024);
    ///
    /// // This many elements can be prepended without invalidating pointers.
    /// let ptr = instance.as_ptr();
    /// for element in 0..instance.capacity_front() {
    ///     assert!(instance.prepend(element).is_ok()) // Cannot fail.
    /// }
    /// assert_eq!(instance.as_ptr(), ptr)
    /// ```
    #[must_use]
    pub fn capacity_front(&self) -> usize {
        if self.initialized == 0 {
            self.capacity()
        } else {
            self.front_capacity
        }
    }

    /// How many elements can [`Self::append`] in without reallocation.
    ///
    /// This many end-specific insertions will be constant time without
    /// possibility of error. Moreover, this maintains pointer validity
    /// even to specific elements.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// // Constructing with generic capacity.
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    /// assert_eq!(instance.capacity_back(), 256);
    ///
    /// // Reserving for specific end of the buffer.
    /// instance.reserve_back(512).expect("successful allocation");
    /// assert_eq!(instance.capacity_back(), 512);
    ///
    /// // Reserving for wrong end of the buffer, but be empty.
    /// instance.reserve_front(1024).expect("successful allocation");
    /// assert_eq!(instance.capacity_back(), 1024);
    ///
    /// // That many elements can be appended without invalidating pointers.
    /// let ptr = instance.as_ptr();
    /// for element in 0..instance.capacity_back() {
    ///     assert!(instance.append(element).is_ok()) // Cannot fail.
    /// }
    /// assert_eq!(instance.as_ptr(), ptr)
    /// ```
    #[must_use]
    pub fn capacity_back(&self) -> usize {
        if self.initialized == 0 {
            self.capacity()
        } else {
            self.back_capacity
        }
    }

    /// Allocate space for _at least_ `capacity` additional elements.
    ///
    /// This method emulates the behaviour of Rust's [`Vec::reserve`].
    ///
    /// In contrast to [`Self::reserve_back`], this method will shift the
    /// initialized elements to consume [`Self::capacity_front`] (thereby
    /// making it zero) before (re)allocating additional
    /// [`Self::capacity_back`] if necessary to have at least `capacity`.
    ///
    /// Furthermore, this method increases the size of buffer by a geometric
    /// progression with a growth factor of two (2), hence the buffer could
    /// ideally contain a power of two (2) number of elements. This means it
    /// may allocate more memory than explicitly requested, but will attempt
    /// to recover when exactly `capacity` can be allocated, but not more. This
    /// means you can apply
    /// [amortized analysis](https://en.wikipedia.org/wiki/Amortized_analysis).
    ///
    /// See also: [`Self::reserve_front`] or [`Self::reserve_back`] to reserve
    /// an exact amount of elements at a specific end of the buffer whilst
    /// preserving existing capacity at the other end.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// // Reclaims front capacity before reallocation.
    /// instance.reserve_front(256).expect("successful allocation");
    /// assert!(instance.reserve(256).is_ok()); // Cannot fail.
    /// assert_eq!(instance.capacity_back(), 256); // Reuses the allocation.
    ///
    /// // Will allocate additional memory if needed.
    /// instance.reserve(512).expect("successful allocation");
    /// assert_eq!(instance.capacity_back(), 1018); // Not 512 - 6! Amortized!
    ///
    /// // That many elements can be inserted without invalidating pointers.
    /// let ptr = instance.as_ptr();
    /// for element in 0..instance.capacity_back() {
    ///     assert!(instance.append(element).is_ok()) // Cannot fail.
    /// }
    /// assert_eq!(instance.as_ptr(), ptr);
    /// ```
    pub fn reserve(&mut self, capacity: usize) -> Result<&mut Self, FailedAllocation> {
        // Reclaim any front capacity.
        if self.initialized > 0 {
            let Ok(offset) = isize::try_from(self.front_capacity) else {
                unreachable!("allocated more than `isize::MAX` bytes");
            };

            let Some(offset) = offset.checked_neg() else {
                unreachable!("negative amount of front capacity");
            };

            let Ok(_) = self.shift(offset) else {
                unreachable!("not enough front capacity to shift into");
            };

            if let Some(total) = self.back_capacity.checked_add(self.front_capacity) {
                self.front_capacity = 0;
                self.back_capacity = total;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes");
            }
        }

        // Prevent amortized growth when unnecessary.
        if self.back_capacity >= capacity {
            return Ok(self);
        }

        let amortized = self.amortized(capacity).unwrap_or(capacity);

        if self.reserve_back(amortized).is_ok() {
            Ok(self)
        } else {
            self.reserve_back(capacity)
        }
    }

    /// Allocate space for exactly `capacity` elements to be prepended.
    ///
    /// If this is okay, that many element can be prepended in constant time
    /// without possibility of error. Moreover, this maintains pointer validity
    /// even to specific elements.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::default();
    ///
    /// instance.reserve_front(256).expect("successful allocation");
    /// assert_eq!(instance.capacity_front(), 256);
    ///
    /// // That many elements can be prepended without invalidating pointers.
    /// let ptr = instance.as_ptr();
    /// for element in 0..instance.capacity_front() {
    ///     assert!(instance.prepend(element).is_ok()) // Cannot fail.
    /// }
    /// assert_eq!(instance.as_ptr(), ptr);
    /// ```
    pub fn reserve_front(&mut self, capacity: usize) -> Result<&mut Self, FailedAllocation> {
        let Some(capacity) = capacity.checked_sub(self.capacity_front()) else {
            debug_assert!(self.capacity_front() > capacity, "enough capacity");

            return Ok(self);
        };

        let capacity = isize::try_from(capacity).map_err(|_| FailedAllocation)?;

        _ = self.resize(capacity)?;

        if self.initialized > 0 {
            let Ok(_) = self.shift(capacity) else {
                unreachable!("not enough back capacity to shift into");
            };
        }

        Ok(self)
    }

    /// Allocate space for exactly `capacity` elements to be appended.
    ///
    /// If this is okay, that many element can be appended in constant time
    /// without possibility of error. Moreover, this maintains pointer validity
    /// even to specific elements.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::default();
    ///
    /// instance.reserve_back(256).expect("successful allocation");
    /// assert_eq!(instance.capacity_back(), 256);
    ///
    /// // That many elements can be appended without invalidating pointers.
    /// let ptr = instance.as_ptr();
    /// for element in 0..instance.capacity_back() {
    ///     assert!(instance.append(element).is_ok()) // Cannot fail.
    /// }
    /// assert_eq!(instance.as_ptr(), ptr);
    /// ```
    pub fn reserve_back(&mut self, capacity: usize) -> Result<&mut Self, FailedAllocation> {
        let Some(capacity) = capacity.checked_sub(self.capacity_back()) else {
            debug_assert!(self.capacity_back() > capacity, "enough capacity");

            return Ok(self);
        };

        let capacity = isize::try_from(capacity).map_err(|_| FailedAllocation)?;

        self.resize(capacity)
    }

    /// Attempt to reduce capacity to exactly `capacity`, or none/zero.
    ///
    /// This method emulates the behaviour of Rust's [`Vec::shrink_to`].
    ///
    /// In contrast to [`Self::shrink_back`], this method will shift the
    /// initialized elements to consume [`Self::capacity_front`] (thereby
    /// making it zero) before reallocating if necessary to reduce
    /// [`Self::capacity_back`].
    ///
    /// See also: [`Self::shrink_front`] or [`Self::shrink_back`] to shrink a
    /// specific end of the buffer without shifting initialized elements.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    ///
    /// // Half fill with elements.
    /// for element in 0..128 {
    ///     instance.prepend(element).expect("enough capacity");
    /// }
    ///
    /// assert_eq!(instance.capacity_front(), 128);
    /// assert_eq!(instance.capacity_back(), 0);
    ///
    /// // Shrink to have capacity of 128 elements at the back.
    /// instance.shrink(Some(128)).expect("successful reallocation");
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 128);
    ///
    /// // Shrink to have no capacity (shrink to fit).
    /// instance.shrink(None).expect("successful deallocation");
    /// assert_eq!(instance.capacity_back(), 0);
    /// ```
    pub fn shrink(&mut self, capacity: Option<usize>) -> Result<&mut Self, FailedAllocation> {
        let Ok(offset) = isize::try_from(self.front_capacity) else {
            unreachable!("allocated more than `isize::MAX` bytes");
        };

        let Some(offset) = offset.checked_neg() else {
            unreachable!("negative front capacity");
        };

        if self.shift(offset).is_err() {
            unreachable!("not enough front capacity to shift into");
        }

        self.shrink_back(capacity)
    }

    /// Reallocate to reduce [`Self::capacity_front`] to exactly `capacity`.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    ///
    /// // Half fill with elements.
    /// for element in 0..128 {
    ///     instance.prepend(element).expect("enough capacity");
    /// }
    ///
    /// assert_eq!(instance.capacity_front(), 128);
    /// assert_eq!(instance.capacity_back(), 0);
    ///
    /// // Shrink to have capacity of 64 elements at the front.
    /// instance.shrink_front(Some(64)).expect("successful reallocation");
    /// assert_eq!(instance.capacity_front(), 64);
    /// assert_eq!(instance.capacity_back(), 0);
    ///
    /// // Shrink to have no capacity (shrink to fit).
    /// instance.shrink_front(None).expect("successful reallocation");
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 0);
    /// ```
    pub fn shrink_front(&mut self, capacity: Option<usize>) -> Result<&mut Self, FailedAllocation> {
        let capacity = capacity.unwrap_or(0);

        let Some(extra) = self.capacity_front().checked_sub(capacity) else {
            debug_assert!(self.capacity_front() < capacity, "small enough");

            return Ok(self);
        };

        let Ok(extra) = isize::try_from(extra) else {
            unreachable!("allocated more than `isize::MAX` bytes");
        };

        let Some(extra) = extra.checked_neg() else {
            unreachable!("negative extra capacity");
        };

        if self.initialized > 0 {
            let Ok(_) = self.shift(extra) else {
                unreachable!("not enough front capacity to shift into");
            };
        }

        self.resize(extra)
    }

    /// Reallocate to reduce back capacity to exactly `capacity` elements.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise `abort` if allocation fails.
    ///
    /// # Errors
    /// Yields [`FailedAllocation`] when memory (re)allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    /// use rust::structure::collection::linear::List;
    ///
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    ///
    /// // Half fill with elements.
    /// instance.extend(0..128);
    ///
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 128);
    ///
    /// // Shrink to have capacity of 64 elements at the front.
    /// instance.shrink_back(Some(64)).expect("successful reallocation");
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 64);
    ///
    /// // Shrink to have no capacity (shrink to fit).
    /// instance.shrink_back(None).expect("successful reallocation");
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 0);
    /// ```
    pub fn shrink_back(&mut self, capacity: Option<usize>) -> Result<&mut Self, FailedAllocation> {
        let capacity = capacity.unwrap_or(0);

        let Some(extra) = self.capacity_back().checked_sub(capacity) else {
            debug_assert!(self.capacity_back() < capacity, "small enough");

            return Ok(self);
        };

        let Ok(extra) = isize::try_from(extra) else {
            unreachable!("allocated more than `isize::MAX` bytes");
        };

        let Some(extra) = extra.checked_neg() else {
            unreachable!("extra capacity is negative");
        };

        self.resize(extra)
    }

    /// Shift the initialized elements `offset` positions within the buffer.
    ///
    /// This method  maintains the order of initialized elements, but shifts
    /// them thereby converting some portion of the capacity from front to
    /// back, or vice versa.
    ///
    /// # Errors
    /// Yields [`OutOfBounds`] is there is not enough capacity to shift into.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::with_capacity(256).expect("successful allocation");
    ///
    /// // Fill with elements.
    /// instance.extend(0..256);
    ///
    /// // Allocate capacity at both ends.
    /// instance.reserve_front(256).expect("successful allocation");
    /// instance.reserve_back(256).expect("successful allocation");
    ///
    /// // Shift initialized elements to the front of the buffer.
    /// instance.shift(-256).expect("offset <= capacity_front()");
    /// instance.shift(-1).expect_err("offset out of bounds");
    /// assert_eq!(instance.capacity_front(), 0);
    /// assert_eq!(instance.capacity_back(), 512);
    ///
    /// // Shift initialized elements to the end of the buffer.
    /// instance.shift(512).expect("offset <= capacity_back()");
    /// instance.shift(1).expect_err("offset out of bounds");
    /// assert_eq!(instance.capacity_front(), 512);
    /// assert_eq!(instance.capacity_back(), 0);
    /// ```
    pub fn shift(&mut self, offset: isize) -> Result<&mut Self, OutOfBounds> {
        match offset.cmp(&0) {
            core::cmp::Ordering::Less => {
                if let Some(capacity) = self.front_capacity.checked_sub(offset.unsigned_abs()) {
                    self.front_capacity = capacity;
                } else {
                    debug_assert!(
                        offset.unsigned_abs() > self.front_capacity,
                        "not enough capacity to shift into"
                    );

                    return Err(OutOfBounds);
                }

                if let Some(capacity) = self.back_capacity.checked_add(offset.unsigned_abs()) {
                    self.back_capacity = capacity;
                } else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                }
            }
            core::cmp::Ordering::Greater => {
                if let Some(capacity) = self.back_capacity.checked_sub(offset.unsigned_abs()) {
                    self.back_capacity = capacity;
                } else {
                    debug_assert!(
                        offset.unsigned_abs() > self.back_capacity,
                        "not enough capacity to shift into"
                    );

                    return Err(OutOfBounds);
                }

                if let Some(capacity) = self.front_capacity.checked_add(offset.unsigned_abs()) {
                    self.front_capacity = capacity;
                } else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                }
            }
            core::cmp::Ordering::Equal => return Ok(self),
        }

        let destination = self.as_mut_ptr();

        let Some(offset) = offset.checked_neg() else {
            unreachable!("offset out of bounds");
        };

        // SAFETY: offset is in bounds => aligned within the allocated object.
        let source = unsafe { destination.offset(offset) };

        // SAFETY:
        // * owned memory => source/destination valid for read/writes.
        // * no aliasing restrictions => source and destination can overlap.
        // * underlying buffer is aligned => both pointers are aligned.
        unsafe {
            core::ptr::copy(source, destination, self.initialized);
        }

        Ok(self)
    }

    /// Remove an element by swapping it with the first element.
    ///
    /// In contrast to [`Self::remove`], this method takes constant time and
    /// does _NOT_ preserve order.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.remove_via_front(3), Some(3));
    /// assert_eq!(instance.capacity_front(), 1);
    /// assert_eq!(instance[2], 0);
    /// ```
    #[must_use]
    pub fn remove_via_front(&mut self, index: usize) -> Option<T> {
        if index >= self.initialized {
            return None;
        }

        let front = self.as_mut_ptr();

        // SAFETY: index in bounds => aligned within the allocated object.
        let index = unsafe { front.add(index) };

        // SAFETY:
        // * both pointers are valid for reads and write.
        // * both pointers are aligned.
        // * no aliasing restrictions.
        unsafe {
            core::ptr::swap(front, index);
        }

        // SAFETY:
        // * owned memory => pointer is valid for reads.
        // * Underlying `T` is initialized.
        // * This takes ownership (moved out of the buffer).
        let element = unsafe { front.read() };

        if let Some(decremented) = self.initialized.checked_sub(1) {
            self.initialized = decremented;
        } else {
            unreachable!("no initialized element to remove");
        }

        if let Some(incremented) = self.front_capacity.checked_add(1) {
            self.front_capacity = incremented;
        } else {
            unreachable!("allocated more that `isize::MAX` bytes");
        }

        Some(element)
    }

    /// Remove an element by swapping it with the last element.
    ///
    /// In contrast to [`Self::remove`], this method takes constant time and
    /// does _NOT_ preserve order.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.remove_via_back(3), Some(3));
    /// assert_eq!(instance.capacity_back(), 1);
    /// assert_eq!(instance[3], 5);
    /// ```
    #[must_use]
    pub fn remove_via_back(&mut self, index: usize) -> Option<T> {
        if index >= self.initialized {
            return None;
        }

        let ptr = self.as_mut_ptr();

        let last = {
            let Some(offset) = self.initialized.checked_sub(1) else {
                unreachable!("no initialized element to remove");
            };

            // SAFETY: points to the final element contained.
            unsafe { ptr.add(offset) }
        };

        // SAFETY: index is in bounds => aligned within the allocated object.
        let index = unsafe { ptr.add(index) };

        // SAFETY:
        // * both pointers are valid for reads and write.
        // * both pointers are aligned.
        // * no aliasing restrictions.
        unsafe {
            core::ptr::swap(last, index);
        }

        // SAFETY:
        // * owned memory => pointer is valid for reads.
        // * Underlying `T` is initialized.
        // * This takes ownership (moved out of the buffer).
        let element = unsafe { last.read() };

        if let Some(decremented) = self.initialized.checked_sub(1) {
            self.initialized = decremented;
        } else {
            unreachable!("no initialized element to remove");
        }

        if let Some(incremented) = self.back_capacity.checked_add(1) {
            self.back_capacity = incremented;
        } else {
            unreachable!("allocated more that `isize::MAX` bytes");
        }

        Some(element)
    }

    /// Exactly how much back capacity to allocate to apply amortized analysis.
    ///
    /// See also: [amortized analysis][amortized] and [dynamic array application][dynamic].
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// [amortized]: https://en.wikipedia.org/wiki/Amortized_analysis
    /// [dynamic]: https://en.wikipedia.org/wiki/Dynamic_array#Geometric_expansion_and_amortized_cost
    #[must_use]
    fn amortized(&self, capacity: usize) -> Option<usize> {
        let Some(retained) = self.front_capacity.checked_add(self.initialized) else {
            unreachable!("allocated more the `isize::MAX` bytes");
        };

        let total = retained.checked_add(capacity)?;

        let total = total.checked_next_power_of_two()?;

        total.checked_sub(retained)
    }

    /// Shift the elements within `range` left or right by `offset`.
    ///
    /// Note this does _NOT_ modify internal capacity state.
    ///
    /// # Safety
    /// The `range` must be within bounds, even when shifted by `offset`.
    ///
    /// # Panics
    /// This method has the precondition the start bound is before the end.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    #[inline]
    unsafe fn shift_range(&mut self, range: impl core::ops::RangeBounds<usize>, offset: isize) {
        let start = match range.start_bound() {
            core::ops::Bound::Unbounded => 0,
            core::ops::Bound::Included(start) => *start,
            core::ops::Bound::Excluded(start) => start.saturating_add(1),
        };

        let end = match range.end_bound() {
            core::ops::Bound::Unbounded => self.len(),
            core::ops::Bound::Included(end) => end.saturating_add(1),
            core::ops::Bound::Excluded(end) => *end,
        };

        let Some(elements) = end.checked_sub(start) else {
            panic!("range had end index before start index")
        };

        // SAFETY: points to the where the first initialized element goes.
        let ptr = unsafe { self.buffer.as_ptr().add(self.front_capacity) };

        // SAFETY: caller promises this will stay in bounds.
        let source = unsafe { ptr.add(start) };

        // SAFETY: caller promises this will stay in bounds.
        let destination = unsafe { source.offset(offset) };

        // SAFETY:
        // * start/end in bounds => source/destination valid for read/write.
        // * ranges can overlap => no aliasing restrictions.
        unsafe {
            core::ptr::copy(source, destination, elements);
        }
    }

    /// (Re)allocate the buffer to modify back capacity by `capacity`.
    ///
    /// This method will increase back capacity by `capacity` if positive,
    /// and decrease by `capacity` if negative, (re)allocating if necessary.
    ///
    /// Note that failed allocation will _NOT_ modify the underlying buffer.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    #[inline]
    fn resize(&mut self, capacity: isize) -> Result<&mut Self, FailedAllocation> {
        let capacity = self
            .capacity_back()
            .checked_add_signed(capacity)
            .ok_or(FailedAllocation)?;

        // Zero-size types do _NOT_ occupy memory, so no (re/de)allocation.
        if size_of::<T>() == 0 {
            // Global allocator API limits allocation to `isize:MAX` bytes.
            if capacity > isize::MAX as usize {
                return Err(FailedAllocation);
            }

            self.back_capacity = capacity;

            return Ok(self);
        }

        let Some(unchanged) = self.front_capacity.checked_add(self.initialized) else {
            unreachable!("allocated more than `isize::MAX` bytes");
        };

        let new = {
            let total = unchanged.checked_add(capacity).ok_or(FailedAllocation)?;

            match core::alloc::Layout::array::<T>(total) {
                Ok(layout) => layout,
                Err(_) => return Err(FailedAllocation),
            }
        };

        let Some(total) = unchanged.checked_add(self.back_capacity) else {
            unreachable!("allocated more than `isize::MAX` bytes");
        };

        let ptr = {
            // No previous allocation exists, so create one.
            if total == 0 {
                if new.size() > 0 {
                    // SAFETY: layout has non-zero size.
                    unsafe { alloc::alloc::alloc(new) }.cast::<T>()
                } else {
                    debug_assert_eq!(capacity, 0, "otherwise occupies memory");

                    // empty => The pointer will _NOT_ be read/written to.
                    NonNull::<T>::dangling().as_ptr()
                }
            }
            // Modify an existing buffer allocation.
            else {
                let Ok(existing) = core::alloc::Layout::array::<T>(total) else {
                    return Err(FailedAllocation);
                };

                let ptr = self.buffer.as_ptr().cast::<u8>();

                // Deallocate.
                if unchanged == 0 && capacity == 0 {
                    // SAFETY:
                    // * allocated using the corresponding allocator.
                    // * `existing_layout` is currently allocated.
                    // * `new_layout` has non-zero size.
                    // * `Layout` => `new.size() <= isize::MAX`.
                    unsafe {
                        alloc::alloc::dealloc(ptr, existing);
                    }

                    // empty state => will _NOT_ be read/written to.
                    NonNull::<T>::dangling().as_ptr()
                }
                // Reallocate.
                else {
                    // SAFETY:
                    // * allocated using the corresponding allocator.
                    // * `existing_layout` is currently allocated.
                    // * `new_layout` has non-zero size.
                    // * `Layout` => `new.size() <= isize::MAX`.
                    unsafe { alloc::alloc::realloc(ptr, existing, new.size()) }.cast::<T>()
                }
            }
        };

        // `MaybeUninit<T>` has the same layout as `T`.
        let ptr = ptr.cast::<MaybeUninit<T>>();

        self.buffer = match NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => return Err(FailedAllocation),
        };

        self.back_capacity = capacity;

        Ok(self)
    }
}

impl<T> Drop for Dynamic<T> {
    /// Drops the elements that are initialized and deallocates memory.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// instance.next();      // Consumes the element with value `0`.
    /// instance.next_back(); // Consumes the element with value `5`.
    ///
    /// core::mem::drop(instance); // Drops the elements with values `[1, 2, 3, 4]`.
    /// ```
    fn drop(&mut self) {
        for index in 0..self.initialized {
            let ptr = self.buffer.as_ptr();

            // SAFETY: stays aligned within the allocated object.
            let ptr = unsafe { ptr.add(self.front_capacity) };

            // SAFETY: index is within bounds, so within allocated object.
            let ptr = unsafe { ptr.add(index) };

            // SAFETY: the `MaybeUninit<T>` is initialized.
            let element = unsafe { &mut *ptr };

            // SAFETY: The `T` is initialized => safe drop.
            unsafe {
                element.assume_init_drop();
            }
        }

        if let Some(capacity) = self.back_capacity.checked_add(self.initialized) {
            self.back_capacity = capacity;
            self.initialized = 0;
        } else {
            unreachable!("allocated more than `isize::MAX` bytes");
        }

        let Ok(_) = self.shrink(None) else {
            unreachable!("deallocation failure");
        };
    }
}

impl<'a, T: 'a + Clone> TryFrom<&'a [T]> for Dynamic<T> {
    type Error = FailedAllocation;

    /// Construct by cloning elements from an existing slice.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");
    ///
    /// assert!(actual.eq(expected));
    /// ```
    fn try_from(slice: &'a [T]) -> Result<Self, Self::Error> {
        let mut instance = Self::with_capacity(slice.len())?;

        instance.extend(slice.iter().cloned());

        Ok(instance)
    }
}

impl<T> core::ops::Index<usize> for Dynamic<T> {
    type Output = T;

    /// Query the initialized element `index` positions from the start.
    ///
    /// # Panics
    /// This method has the precondition that the `index` is within bounds.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Dynamic::from_iter(expected.iter().copied());
    ///
    /// for index in 0..expected.len() {
    ///     use core::ops::Index;
    ///     assert_eq!(actual.index(index), expected.index(index));
    /// }
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.initialized, "index out of bounds");

        let ptr = self.as_ptr();

        // SAFETY: index within bounds => stays within the allocated object.
        let ptr = unsafe { ptr.add(index) };

        // SAFETY:
        // * the underlying `T` is initialized.
        // * lifetime bound to self => valid lifetime to return.
        unsafe { &*ptr }
    }
}

impl<T> core::ops::IndexMut<usize> for Dynamic<T> {
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
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let mut actual = Dynamic::from_iter(expected.iter().copied());
    ///
    /// for index in 0..expected.len() {
    ///     use core::ops::IndexMut;
    ///     assert_eq!(actual.index_mut(index), expected.index_mut(index));
    /// }
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.initialized, "index out of bounds");

        let ptr = self.as_mut_ptr();

        // SAFETY: index within bounds => stays within the allocated object.
        let ptr = unsafe { ptr.add(index) };

        // SAFETY:
        // * the underlying `T` is initialized.
        // * lifetime bound to self => valid lifetime to return.
        unsafe { &mut *ptr }
    }
}

impl<T> Iterator for Dynamic<T> {
    type Item = T;

    /// Obtain the first initialized element.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
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
        (self.initialized > 0).then(|| {
            let element = self.as_mut_ptr();

            if let Some(decremented) = self.initialized.checked_sub(1) {
                self.initialized = decremented;
            } else {
                unreachable!("no initialized element to remove");
            };

            if let Some(incremented) = self.front_capacity.checked_add(1) {
                self.front_capacity = incremented;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes");
            };

            // SAFETY:
            // * owned memory => pointer is valid for reads.
            // * Underlying `T` is initialized.
            // * This takes ownership (moved out of the buffer).
            unsafe { element.read() }
        })
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
    ///
    /// assert_eq!(instance.size_hint(), (6, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.initialized, Some(self.initialized))
    }
}

impl<T> DoubleEndedIterator for Dynamic<T> {
    /// Obtain the last initialized element.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]).into_iter();
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
        (self.initialized > 0).then(|| {
            if let Some(decremented) = self.initialized.checked_sub(1) {
                self.initialized = decremented;
            } else {
                unreachable!("no initialized element to remove");
            }

            if let Some(incremented) = self.back_capacity.checked_add(1) {
                self.back_capacity = incremented;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes");
            };

            let ptr = self.as_mut_ptr();

            // SAFETY: final initialized element in the allocated object.
            let element = unsafe { ptr.add(self.initialized) };

            // SAFETY:
            // * owned memory => pointer is valid for reads.
            // * Underlying `T` is initialized.
            // * This takes ownership (moved out of the buffer).
            unsafe { element.read() }
        })
    }
}

impl<T> ExactSizeIterator for Dynamic<T> {}

impl<T> core::iter::FusedIterator for Dynamic<T> {}

impl<T> FromIterator<T> for Dynamic<T> {
    /// Construct by moving elements from an iterator.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let actual: Dynamic<_> = expected.clone().into_iter().collect();
    ///
    /// assert!(actual.eq(expected))
    /// ```
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let iter = iter.into_iter();

        let mut instance = Self::default();

        instance.extend(iter);

        instance
    }
}

impl<T> Extend<T> for Dynamic<T> {
    /// Append elements of an iterator in order.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    ///
    /// let mut instance = Dynamic::<i32>::default();
    ///
    /// instance.extend(expected.iter().cloned());
    ///
    /// assert!(instance.eq(expected))
    /// ```
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        let iter = iter.into_iter();

        // `size_hint` can _NOT_ be trusted to exact size.
        let count = {
            let (min, max) = iter.size_hint();
            max.unwrap_or(min)
        };

        // Append will allocate for each realized element reserve if fails.
        drop(self.reserve_back(count));

        for element in iter {
            assert!(self.append(element).is_ok(), "allocation failed");
        }
    }
}

impl<T> Default for Dynamic<T> {
    /// Construct an instance with no elements and no capacity/allocation.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let instance = Dynamic::<()>::default();
    ///
    /// assert_eq!(instance.len(), 0);
    /// assert_eq!(instance.capacity(), 0);
    /// ```
    fn default() -> Self {
        Self {
            buffer: NonNull::dangling(),
            front_capacity: 0,
            initialized: 0,
            back_capacity: 0,
        }
    }
}

impl<T: Clone> Clone for Dynamic<T> {
    /// Construct an instance with no elements and no capacity/allocation.
    ///
    /// # Panics
    /// The Rust runtime might abort if allocation fails, panics otherwise.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(expected.clone(), expected)
    /// ```
    fn clone(&self) -> Self {
        let mut clone = Self::default();

        clone.extend(self.iter().cloned());

        clone
    }
}

impl<T: PartialEq> PartialEq for Dynamic<T> {
    /// Query if the elements contained are the same as `other`.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let left = [0, 1, 2, 3, 4, 5];
    /// let right = left.clone();
    ///
    /// let left = Dynamic::from_iter(left);
    /// let right = Dynamic::from_iter(right);
    ///
    /// assert_eq!(left, right);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for Dynamic<T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for Dynamic<T> {
    /// List the elements contained.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Dynamic::from_iter(expected.iter());
    ///
    /// assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Collection for Dynamic<T> {
    type Element = T;

    /// Query the number of initialized elements contained.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::Collection;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let instance = Dynamic::from_iter(expected.clone());
    ///
    /// assert_eq!(Collection::count(&instance), expected.len());
    /// ```
    fn count(&self) -> usize {
        self.initialized
    }
}

impl<T> Linear for Dynamic<T> {
    /// Create an immutable iterator over the initialized elements.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let expected = [0, 1, 2, 3, 4, 5];
    /// let actual = Dynamic::from_iter(expected.clone());
    ///
    /// for (actual, expected) in actual.iter().zip(expected.iter()) {
    ///     assert_eq!(actual, expected);
    /// }
    /// ```
    fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &Self::Element> + ExactSizeIterator + core::iter::FusedIterator
    {
        let ptr = if self.initialized > 0 {
            // The pointer will only ever be read, no written to.
            let ptr = self.as_ptr().cast_mut();

            // SAFETY: initialized elements => `ptr` is non-null
            unsafe { NonNull::new_unchecked(ptr) }
        } else {
            debug_assert_eq!(self.initialized, 0, "initialized elements");

            // no initialized elements => The pointer will not be read.
            NonNull::dangling()
        };

        // SAFETY: `ptr` is dangling if and only if no elements have been
        // initialized, in which case the pointer will not be read.
        unsafe { super::Iter::new(ptr, self.initialized) }
    }

    /// Create a mutable iterator over the initialized elements.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::Linear;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut expected = [0, 1, 2, 3, 4, 5];
    /// let mut actual = Dynamic::from_iter(expected.clone());
    ///
    /// for (actual, expected) in actual.iter_mut().zip(expected.iter_mut()) {
    ///     assert_eq!(actual, expected);
    /// }
    /// ```
    fn iter_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &mut Self::Element>
    + ExactSizeIterator
    + core::iter::FusedIterator {
        let ptr = if self.initialized > 0 {
            let ptr = self.as_mut_ptr();

            // SAFETY: initialized elements => `ptr` is non-null
            unsafe { NonNull::new_unchecked(ptr) }
        } else {
            debug_assert_eq!(self.initialized, 0, "initialized elements");

            // no initialized elements => The pointer will not be read.
            NonNull::dangling()
        };

        // SAFETY: `ptr` is dangling if and only if no elements have been
        // initialized, in which case the pointer will not be read.
        unsafe { super::IterMut::new(ptr, self.initialized) }
    }
}

impl<T> Array for Dynamic<T> {
    /// Obtain an immutable pointer to the underlying contigious memory buffer.
    ///
    /// The pointer starts at the first initialized element.
    ///
    /// # Safety
    /// * `self` must outlive the pointer.
    /// * The pointer must never be written to.
    /// * Modifying `self` might invalidate the pointer.
    ///
    /// # Panics
    /// This method has the precondition that an underlying allocation exists
    /// to point to. Note that a dangling (but nevertheless valid) pointer will
    /// be yielded for zero-size types despite not occupying memory.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// let expected = core::ptr::from_ref(&instance[0]);
    /// let actual = unsafe { instance.as_ptr() };
    ///
    /// assert_eq!(actual, expected);
    /// ```
    fn as_ptr(&self) -> *const Self::Element {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "we cannot allocate more than `usize::MAX`"
        )]
        let allocated = self.front_capacity + self.initialized + self.back_capacity;

        assert!(allocated > 0, "no allocation to point to");

        // `MaybeUninit<T>` has the same layout as `T`.
        let ptr = self.buffer.cast::<T>().as_ptr().cast_const();

        // SAFETY: Stays aligned within the allocated object.
        unsafe { ptr.add(self.front_capacity) }
    }

    /// Obtain a mutable pointer to the underlying contigious memory buffer.
    ///
    /// The pointer starts at the first initialized element.
    ///
    /// # Safety
    /// * `self` must outlive the pointer.
    /// * Modifying `self` might invalidate the pointer.
    ///
    /// # Panics
    /// This method has the precondition that an underlying allocation exists
    /// to point to. Note that a dangling (but nevertheless valid) pointer will
    /// be yielded for zero-size types despite not occupying memory.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Array;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// let expected = core::ptr::from_ref(&instance[0]).cast_mut();
    /// let actual = unsafe { instance.as_mut_ptr() };
    ///
    /// assert_eq!(actual, expected);
    /// ```
    fn as_mut_ptr(&mut self) -> *mut Self::Element {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "we cannot allocate more than `usize::MAX`"
        )]
        let allocated = self.front_capacity + self.initialized + self.back_capacity;

        assert!(allocated > 0, "no allocation to point to");

        // `MaybeUninit<T>` has the same layout as `T`.
        let ptr = self.buffer.cast::<T>().as_ptr();

        // SAFETY: Stays aligned within the allocated object.
        unsafe { ptr.add(self.front_capacity) }
    }
}

impl<T> List for Dynamic<T> {
    /// Insert an `element` at `index`.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::default();
    ///
    /// instance.insert(0, 1);
    /// instance.insert(1, 3);
    /// instance.insert(1, 2);
    /// instance.insert(0, 0);
    ///
    /// assert!(instance.into_iter().eq([0, 1, 2, 3]));
    /// ```
    fn insert(
        &mut self,
        index: usize,
        element: Self::Element,
    ) -> Result<&mut Self::Element, Self::Element> {
        if index > self.initialized {
            return Err(element);
        }

        let mut ptr = self.buffer.as_ptr();

        // consume front capacity.
        if index == 0 && self.capacity_front() > 0 {
            ptr = {
                let Some(offset) = self.capacity_front().checked_sub(1) else {
                    unreachable!("zero front capacity")
                };

                // SAFETY: the last uninitialized element in the front.
                unsafe { ptr.add(offset) }
            };

            // Shift all capacity to front capacity.
            if self.initialized == 0 {
                if let Some(capacity) = self.front_capacity.checked_add(self.back_capacity) {
                    self.front_capacity = capacity;
                } else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                };

                self.back_capacity = 0;
            }

            if let Some(decremented) = self.front_capacity.checked_sub(1) {
                self.front_capacity = decremented;
            } else {
                unreachable!("no front capacity to insert into");
            };
        }
        // consume back capacity.
        else if self.reserve(1).is_ok() {
            ptr = {
                let Some(offset) = self.front_capacity.checked_add(index) else {
                    unreachable!("index out of bounds");
                };

                // SAFETY: the uninitialized element to insert into.
                unsafe { self.buffer.as_ptr().add(offset) }
            };

            // SAFETY: there is back capacity to shift into.
            unsafe {
                self.shift_range(index.., 1);
            }

            if let Some(decrement) = self.back_capacity.checked_sub(1) {
                self.back_capacity = decrement;
            } else {
                unreachable!("no back capacity to insert into");
            };
        } else {
            debug_assert_eq!(self.capacity(), 0, "no capacity to insert into");

            return Err(element);
        }

        if let Some(increment) = self.initialized.checked_add(1) {
            self.initialized = increment;
        } else {
            unreachable!("allocated more that `isize::MAX` bytes");
        };

        // SAFETY: the `MaybeUninit<T>` is initialized even if the `T` isn't.
        let uninit_element = unsafe { &mut *ptr };

        // the underlying `T` is unutilized.
        Ok(uninit_element.write(element))
    }

    /// Remove the element at `index`.
    ///
    /// # Performance
    /// This methods takes O(N) time and O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0,1,2,3,4,5]);
    ///
    /// instance.remove(5);
    /// instance.remove(2);
    /// instance.remove(0);
    ///
    /// assert!(instance.into_iter().eq([1, 3, 4]));
    /// ```
    fn remove(&mut self, index: usize) -> Option<Self::Element> {
        if index >= self.initialized {
            return None;
        }

        let element = {
            // SAFETY: index within bounds => aligned within allocated object.
            let ptr = unsafe { self.as_ptr().add(index) };

            // SAFETY:
            // * owned memory => pointer is valid for reads.
            // * Underlying `T` is initialized.
            // * This takes ownership (moved out of the buffer).
            unsafe { ptr.read() }
        };

        // Increase front capacity.
        if index == 0 {
            if let Some(incremented) = self.front_capacity.checked_add(1) {
                self.front_capacity = incremented;
            } else {
                unreachable!("allocated more that `isize::MAX` bytes");
            };
        }
        // Increase back capacity.
        else {
            // SAFETY: there is back capacity to shift into.
            unsafe {
                self.shift_range(index.saturating_add(1).., -1);
            }

            if let Some(incremented) = self.back_capacity.checked_add(1) {
                self.back_capacity = incremented;
            } else {
                unreachable!("allocated more that `isize::MAX` bytes");
            };
        }

        if let Some(decremented) = self.initialized.checked_sub(1) {
            self.initialized = decremented;
        } else {
            unreachable!("no initialized element to remove");
        };

        Some(element)
    }

    /// Optimally remove elements within `range` by-value.
    ///
    /// This method is more efficient than using `remove` for sequential
    /// elements, moving elements out of the buffer as iterated and shifting
    /// once only when the iterator has been dropped.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    /// use rust::structure::collection::linear::List;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5, 6, 7]);
    ///
    /// let mut drain = instance.drain(..2);
    /// assert_eq!(drain.next(), Some(0));
    /// assert_eq!(drain.next_back(), Some(1));
    /// core::mem::drop(drain);
    ///
    /// let mut drain = instance.drain(0..2);
    /// assert_eq!(drain.next(), Some(2));
    /// assert_eq!(drain.next_back(), Some(3));
    /// core::mem::drop(drain);
    ///
    /// let mut drain = instance.drain(0..=1);
    /// assert_eq!(drain.next(), Some(4));
    /// assert_eq!(drain.next_back(), Some(5));
    /// core::mem::drop(drain);
    ///
    /// let mut drain = instance.drain(0..);
    /// assert_eq!(drain.next(), Some(6));
    /// assert_eq!(drain.next_back(), Some(7));
    /// core::mem::drop(drain);
    ///
    /// let mut drain = instance.drain(..);
    /// assert_eq!(drain.next(), None);
    /// assert_eq!(drain.next_back(), None);
    /// ```
    fn drain(
        &mut self,
        range: impl core::ops::RangeBounds<usize>,
    ) -> impl DoubleEndedIterator<Item = Self::Element> + ExactSizeIterator {
        let start = match range.start_bound() {
            core::ops::Bound::Included(start) => *start,
            core::ops::Bound::Excluded(start) => start.saturating_add(1),
            core::ops::Bound::Unbounded => 0,
        }
        .min(self.len());

        let end = match range.end_bound() {
            core::ops::Bound::Included(end) => end.saturating_add(1),
            core::ops::Bound::Excluded(end) => *end,
            core::ops::Bound::Unbounded => self.len(),
        }
        .min(self.len());

        let normalized = start..end;

        Drain {
            underlying: self,
            range: normalized.clone(),
            next: normalized.clone(),
        }
    }

    /// Remove the elements which match some `predicate`.
    ///
    /// The `predicate` is called exactly once per each element, in order of
    /// iteration. Elements for which the `predicate` is true are removed in
    /// order from left to right. Elements for which the `predicate` is false
    /// are shifted left to immediately after the previously retained element,
    /// thereby maintaining order.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory for the result.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    /// use rust::structure::collection::linear::List;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut withdraw = instance.withdraw(|element| element % 2 == 0);
    ///
    /// assert_eq!(withdraw.next(), Some(0));
    /// assert_eq!(withdraw.next_back(), Some(4));
    ///
    /// drop(withdraw);
    ///
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn withdraw(
        &mut self,
        predicate: impl FnMut(&T) -> bool,
    ) -> impl DoubleEndedIterator<Item = Self::Element> {
        let head = if self.initialized == 0 {
            // is empty => this pointer will _NOT_ be modified or read.
            NonNull::dangling()
        } else {
            // SAFETY: at least one element exist => pointer cannot be null.
            unsafe { NonNull::new_unchecked(self.as_mut_ptr()) }
        };

        let tail = {
            let ptr = {
                let offset = self.initialized.saturating_sub(1);

                // SAFETY: stays aligned within the allocated object.
                unsafe { head.as_ptr().add(offset) }
            };

            // SAFETY: `head` cannot be null => pointer cannot be null.
            unsafe { NonNull::new_unchecked(ptr) }
        };

        let remaining = self.initialized;

        Withdraw {
            underlying: self,
            predicate,
            remaining,
            retained: head,
            next_front: head,
            next_back: tail,
            trailing: 0,
        }
    }

    /// Drop all initialized elements
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0,1,2,3,4,5]);
    ///
    /// instance.clear();
    ///
    /// assert_eq!(instance.len(), 0);
    /// assert_eq!(instance.capacity(), 6);
    /// ```
    fn clear(&mut self) {
        if self.initialized == 0 {
            return;
        }

        let ptr = self.as_mut_ptr().cast::<MaybeUninit<T>>();

        for index in 0..self.initialized {
            // SAFETY: index in bounds => aligned within the allocated object.
            let ptr = unsafe { ptr.add(index) };

            // SAFETY: the `MaybeUninit<T>` is initialized.
            let element = unsafe { &mut *ptr };

            // SAFETY: the underlying `T` is initialized.
            unsafe {
                element.assume_init_drop();
            }
        }

        if let Some(capacity) = self.back_capacity.checked_add(self.initialized) {
            self.back_capacity = capacity;
        } else {
            unreachable!("allocated more than `isize::MAX` bytes");
        }

        self.initialized = 0;
    }
}

impl<T> super::super::Stack for Dynamic<T> {
    /// Move an `element` on the top of the stack.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::default();
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
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
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
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.peek(), Some(&0));
    /// ```
    fn peek(&self) -> Option<&Self::Element> {
        self.first()
    }
}

impl<T> super::super::Queue for Dynamic<T> {
    /// Move an `element` to the end of the queue.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::<usize>::default();
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
        self.append(element)
    }

    /// Move out the element at the front of the queue.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
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
    /// use rust::structure::collection::linear::Stack;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(instance.peek(), Some(&0));
    /// ```
    fn peek(&self) -> Option<&Self::Element> {
        self.first()
    }
}

/// [`Iterator`] to yield elements within an index range from [`Dynamic`].
///
/// See [`Dynamic::drain`].
struct Drain<'a, T> {
    /// The underlying [`Dynamic`] being drained from.
    underlying: &'a mut Dynamic<T>,

    /// The index range of elements being drained.
    range: core::ops::Range<usize>,

    /// The index range of elements being drained that have yet to be yielded.
    next: core::ops::Range<usize>,
}

impl<T> Drop for Drain<'_, T> {
    /// Drops remaining elements and fixes the underlying [`Dynamic`] buffer.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5, 6]);
    ///
    /// let mut drain = instance.drain(2..=4);
    ///
    /// drain.next();      // Consumes the element with value `2`.
    /// drain.next_back(); // Consumes the element with value `4`.
    ///
    /// core::mem::drop(drain); // Drops the element with value '3'.
    ///
    /// assert!(instance.into_iter().eq([0, 1, 5, 6])); // Remaining elements.
    /// ```
    fn drop(&mut self) {
        if self.underlying.initialized == 0 {
            debug_assert_eq!(self.range, 0..0, "drained uninitialized elements");
            return;
        }

        self.for_each(drop);

        if self.range.end == self.underlying.initialized {
            if let Some(capacity) = self.underlying.back_capacity.checked_add(self.range.len()) {
                self.underlying.back_capacity = capacity;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes");
            }
        } else if self.range.start == 0 {
            if let Some(capacity) = self.underlying.front_capacity.checked_add(self.range.len()) {
                self.underlying.front_capacity = capacity;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes");
            }
        } else {
            let leading = self.range.start;

            let Some(trailing) = self.underlying.initialized.checked_sub(self.range.end) else {
                unreachable!("not enough initialized elements to remove");
            };

            let Ok(offset) = isize::try_from(self.range.len()) else {
                unreachable!("allocated more than `isize::MAX` bytes");
            };

            let only_front_capacity =
                self.underlying.front_capacity != 0 && self.underlying.back_capacity == 0;
            let only_back_capacity =
                self.underlying.front_capacity == 0 && self.underlying.back_capacity != 0;

            if only_front_capacity || (!only_back_capacity && trailing > leading) {
                // SAFETY: [front capacity] [shift] [drained] [remain] [back capacity]
                unsafe {
                    self.underlying.shift_range(0..self.range.start, offset);
                }

                let Some(capacity) = self.underlying.front_capacity.checked_add(self.range.len())
                else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                };

                self.underlying.front_capacity = capacity;
            } else {
                let Some(offset) = offset.checked_neg() else {
                    unreachable!("offset was a positive number of elements");
                };

                // SAFETY: [front capacity] [remain] [drained] [shift] [back capacity]
                unsafe {
                    self.underlying
                        .shift_range(self.range.end..self.underlying.initialized, offset);
                }

                let Some(capacity) = self.underlying.back_capacity.checked_add(self.range.len())
                else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                };

                self.underlying.back_capacity = capacity;
            }
        }

        if let Some(decreased) = self.underlying.initialized.checked_sub(self.range.len()) {
            self.underlying.initialized = decreased;
        }
    }
}

impl<T> Iterator for Drain<'_, T> {
    type Item = T;

    /// Obtain the next element, if there are any left.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut actual = underlying.drain(..);
    ///
    /// assert_eq!(actual.next(), Some(0));
    /// assert_eq!(actual.next(), Some(1));
    /// assert_eq!(actual.next(), Some(2));
    /// assert_eq!(actual.next_back(), Some(5));
    /// assert_eq!(actual.next_back(), Some(4));
    /// assert_eq!(actual.next_back(), Some(3));
    /// assert_eq!(actual.next(), None);
    /// assert_eq!(actual.next_back(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        self.next.next().map_or_else(
            || None,
            |index| {
                let ptr = self.underlying.as_mut_ptr().cast::<MaybeUninit<T>>();

                // SAFETY: stays aligned within the allocated object.
                let ptr = unsafe { ptr.add(index) };

                // SAFETY: index in bounds => aligned within the allocated object.
                let element = unsafe { &mut *ptr };

                // SAFETY:
                // * owned memory => pointer is valid for reads.
                // * Underlying `T` is initialized.
                // * This takes ownership (moved out of the buffer).
                Some(unsafe { element.assume_init_read() })
            },
        )
    }

    /// Query how many elements have yet to be yielded.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::array::Dynamic;
    /// use rust::structure::collection::linear::List;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut actual = underlying.drain(..);
    ///
    /// assert_eq!(actual.size_hint(), (6, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.next.len(), Some(self.next.len()))
    }
}

impl<T> DoubleEndedIterator for Drain<'_, T> {
    /// Obtain the final element, if there are any left.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut actual = underlying.drain(..);
    ///
    /// assert_eq!(actual.next_back(), Some(5));
    /// assert_eq!(actual.next_back(), Some(4));
    /// assert_eq!(actual.next_back(), Some(3));
    /// assert_eq!(actual.next_back(), Some(2));
    /// assert_eq!(actual.next_back(), Some(1));
    /// assert_eq!(actual.next_back(), Some(0));
    /// assert_eq!(actual.next_back(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next.next_back().map_or_else(
            || None,
            |index| {
                let ptr = self.underlying.as_mut_ptr().cast::<MaybeUninit<T>>();

                // SAFETY: stays aligned within the allocated object.
                let ptr = unsafe { ptr.add(index) };

                // SAFETY: index in bounds => aligned within the allocated object.
                let element = unsafe { &mut *ptr };

                // SAFETY:
                // * owned memory => pointer is valid for reads.
                // * Underlying `T` is initialized.
                // * This takes ownership (moved out of the buffer).
                Some(unsafe { element.assume_init_read() })
            },
        )
    }
}

impl<T> ExactSizeIterator for Drain<'_, T> {}

impl<T> core::iter::FusedIterator for Drain<'_, T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for Drain<'_, T> {
    /// List the elements being drained.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(N) memory.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list = f.debug_list();

        let slice = {
            // SAFETY: index in bounds => aligned within the allocated object.
            let ptr = unsafe { self.underlying.as_ptr().add(self.next.start) };

            // SAFETY: points to yet to be yielded slice.
            unsafe { core::slice::from_raw_parts(ptr, self.next.len()) }
        };

        list.entries(slice).finish()
    }
}

/// [`Iterator`] to yield elements matching a predicate from [`Dynamic`].
///
/// See [`Dynamic::withdraw`].
struct Withdraw<'a, T, F: FnMut(&T) -> bool> {
    /// The underlying [`Dynamic`] begin withdrawn from.
    underlying: &'a mut Dynamic<T>,

    /// The predicate based upon which elements are withdrawn.
    predicate: F,

    /// Where to write the next retained element to.
    retained: NonNull<T>,

    /// How many element are left to query with the predicate.
    remaining: usize,

    /// The next (front) element to query with the predicate.
    next_front: NonNull<T>,

    /// The next (back) element to query with the predicate.
    next_back: NonNull<T>,

    /// The number of retained elements at the end because of `next_back`.
    trailing: usize,
}

impl<T, F: FnMut(&T) -> bool> Drop for Withdraw<'_, T, F> {
    /// Drops remaining elements and fixes the underlying [`Dynamic`] buffer.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut instance = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    ///
    /// let mut withdraw = instance.withdraw(|element| element % 2 == 0);
    ///
    /// // Consumes the element with value `0`.
    /// assert_eq!(withdraw.next(), Some(0));
    ///
    /// // Consumes the element with value `4`.
    /// assert_eq!(withdraw.next_back(), Some(4));
    ///
    /// // Drops the element with value '2'.
    /// drop(withdraw);
    ///
    /// // Retained elements.
    /// assert!(instance.eq([1, 3, 5]));
    /// ```
    fn drop(&mut self) {
        // Drop all remaining elements to withdraw.
        self.for_each(drop);

        if self.trailing > 0 {
            // SAFETY: aligned within the allocated object, or one byte past.
            let trailing = unsafe { self.next_back.as_ptr().add(1) };

            // SAFETY:
            // * owned memory => source/destination valid for read/writes.
            // * no aliasing restrictions => source and destination can overlap.
            // * underlying buffer is aligned => both pointers are aligned.
            unsafe {
                core::ptr::copy(trailing, self.retained.as_ptr(), self.trailing);
            }
        }
    }
}

impl<T, F: FnMut(&T) -> bool> Iterator for Withdraw<'_, T, F> {
    type Item = T;

    /// Obtain the next element, if there are any left.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut actual = underlying.withdraw(|element| element % 2 == 0);
    ///
    /// assert_eq!(actual.next(), Some(0));
    /// assert_eq!(actual.next(), Some(2));
    /// assert_eq!(actual.next(), Some(4));
    /// assert_eq!(actual.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        let first_retained = self.next_front;
        let mut consecutive_retained = 0;

        // SAFETY:
        // * owned memory => source/destination valid for read/writes.
        // * no aliasing restrictions => source and destination can overlap.
        // * underlying buffer is aligned => both pointers are aligned.
        let shift_retained = |src: *mut T, dst: *mut T, count| unsafe {
            // Shift the current run of retained elements to the left.
            core::ptr::copy(src, dst, count);
        };

        while self.remaining != 0 {
            if let Some(remaining) = self.remaining.checked_sub(1) {
                self.remaining = remaining;
            } else {
                unreachable!("no remaining element");
            }

            // SAFETY: the element is initialized.
            let current = unsafe { self.next_front.as_ref() };

            self.next_front = {
                // SAFETY: aligned within the allocated object, or one byte past.
                let ptr = unsafe { self.next_front.as_ptr().add(1) };

                // SAFETY: `head` is not null => pointer is not null.
                unsafe { NonNull::new_unchecked(ptr) }
            };

            if (self.predicate)(current) {
                // SAFETY:
                // * owned memory => pointer is valid for reads.
                // * Underlying `T` is initialized.
                // * This takes ownership (moved out of the buffer).
                let element = unsafe { core::ptr::read(current) };

                if self.underlying.as_ptr() == current {
                    // Will not shift, instead increasing front capacity.
                    if let Some(incremented) = self.underlying.front_capacity.checked_add(1) {
                        self.underlying.front_capacity = incremented;
                    } else {
                        unreachable!("allocated more than `isize::MAX` bytes");
                    }

                    // The current element will be left uninitialized.
                    self.retained = {
                        // SAFETY: at most one byte past the allocated object.
                        let ptr = unsafe { self.retained.as_ptr().add(1) };

                        // SAFETY: `retained` is not null => pointer is not null.
                        unsafe { NonNull::new_unchecked(ptr) }
                    };
                } else {
                    // will shift elements to increase back capacity.
                    if let Some(incremented) = self.underlying.back_capacity.checked_add(1) {
                        self.underlying.back_capacity = incremented;
                    } else {
                        unreachable!("allocated more than `isize::MAX` bytes");
                    }
                }

                shift_retained(
                    first_retained.as_ptr(),
                    self.retained.as_ptr(),
                    consecutive_retained,
                );

                self.retained = {
                    // SAFETY: next uninitialized element, or one byte past.
                    let ptr = unsafe { self.retained.as_ptr().add(consecutive_retained) };

                    // SAFETY: `retained` is not null => pointer is not null.
                    unsafe { NonNull::new_unchecked(ptr) }
                };

                if let Some(decremented) = self.underlying.initialized.checked_sub(1) {
                    self.underlying.initialized = decremented;
                } else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                }

                return Some(element);
            }

            if let Some(incremented) = consecutive_retained.checked_add(1) {
                consecutive_retained = incremented;
            } else {
                unreachable!("allocated more than `isize::MAX` bytes")
            }
        }

        // The above loop will exit whenever there are no more remaining
        // elements to query with the predicate. However, this means the loop
        // may iterate through a string of elements to retain at the end of the
        // buffer before exhausting elements to query. In such a circumstance,
        // there is no element at the end to withdraw hence the loop will exit
        // without shifting these elements to align with previously retained
        // elements. Nevertheless, previous iterations of the loop ensure the
        // pointer and counter denote a valid range of retained elements (if
        // any) so they can still be shifted before returning none.
        shift_retained(
            first_retained.as_ptr(),
            self.retained.as_ptr(),
            consecutive_retained,
        );

        self.retained = {
            // SAFETY: at most one byte past the allocated object.
            let ptr = unsafe { self.retained.as_ptr().add(consecutive_retained) };

            // SAFETY: `retained` is not null => pointer is not null.
            unsafe { NonNull::new_unchecked(ptr) }
        };

        None
    }

    /// Query how many elements can be yielded.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let instance = underlying.withdraw(|element| element % 2 == 0);
    ///
    /// assert_eq!(instance.size_hint(), (0, Some(6)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.remaining))
    }
}

impl<T, F: FnMut(&T) -> bool> DoubleEndedIterator for Withdraw<'_, T, F> {
    /// Obtain the next element, if there are any left.
    ///
    /// # Performance
    /// This methods takes O(N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::structure::collection::linear::List;
    /// use rust::structure::collection::linear::array::Dynamic;
    ///
    /// let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
    /// let mut actual = underlying.withdraw(|element| element % 2 == 0);
    ///
    /// assert_eq!(actual.next_back(), Some(4));
    /// assert_eq!(actual.next_back(), Some(2));
    /// assert_eq!(actual.next_back(), Some(0));
    /// assert_eq!(actual.next_back(), None);
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.remaining != 0 {
            if let Some(decremented) = self.remaining.checked_sub(1) {
                self.remaining = decremented;
            } else {
                unreachable!("no remaining element");
            }

            // SAFETY: the element is initialized.
            let current = unsafe { self.next_back.as_ref() };

            // Do _NOT_ moved the pointer _before_ the allocated object.
            if self.remaining != 0 {
                self.next_back = {
                    // SAFETY: aligned within the allocated object.
                    let ptr = unsafe { self.next_back.as_ptr().sub(1) };

                    // SAFETY: `retained` is not null => pointer is not null.
                    unsafe { NonNull::new_unchecked(ptr) }
                };
            }

            if (self.predicate)(current) {
                // SAFETY:
                // * owned memory => pointer is valid for reads.
                // * Underlying `T` is initialized.
                // * This takes ownership (moved out of the buffer).
                let element = unsafe { core::ptr::read(current) };

                if let Some(decremented) = self.underlying.initialized.checked_sub(1) {
                    self.underlying.initialized = decremented;
                } else {
                    unreachable!("no initialized element to remove");
                }

                if let Some(incremented) = self.underlying.back_capacity.checked_add(1) {
                    self.underlying.back_capacity = incremented;
                } else {
                    unreachable!("allocated more than `isize::MAX` bytes");
                }

                let src = {
                    let current: *const T = current;

                    // SAFETY: stays aligned within the allocated object.
                    unsafe { current.add(1) }.cast_mut()
                };

                let dst = {
                    let current: *const T = current;
                    current.cast_mut()
                };

                // SAFETY:
                // * owned memory => source/destination valid for read/writes.
                // * no aliasing restrictions => source and destination can overlap.
                // * underlying buffer is aligned => both pointers are aligned.
                unsafe {
                    core::ptr::copy(src, dst, self.trailing);
                }

                return Some(element);
            }

            if let Some(incremented) = self.trailing.checked_add(1) {
                self.trailing = incremented;
            } else {
                unreachable!("allocated more than `isize::MAX`");
            };
        }

        None
    }
}

impl<T, F: FnMut(&T) -> bool> core::iter::FusedIterator for Withdraw<'_, T, F> {}

impl<T, F: FnMut(&T) -> bool> core::fmt::Debug for Withdraw<'_, T, F> {
    /// Output what indexes are being pointed to in the underlying buffer.
    ///
    /// Note that these indexes are _NOT_ based on the first initialized
    /// element, but rather absolute relative to the beginning of the
    /// allocated object.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let origin = self.underlying.buffer.as_ptr().cast::<T>();

        // SAFETY: both pointers are aligned within the allocated object.
        let head = unsafe { self.next_front.as_ptr().offset_from(origin) };

        // SAFETY: both pointers are aligned within the allocated object.
        let retained = unsafe { self.retained.as_ptr().offset_from(origin) };

        // SAFETY: both pointers are aligned within the allocated object.
        let tail = unsafe { self.next_back.as_ptr().offset_from(origin) };

        f.debug_struct("Withdraw")
            .field("head index", &head)
            .field("tail index", &tail)
            .field("remaining elements", &self.remaining)
            .field("retained index", &retained)
            .field("trailing elements", &self.trailing)
            .finish_non_exhaustive()
    }
}

/// Error type for recoverable allocation failure.
#[derive(Debug, Clone, Copy)]
pub struct FailedAllocation;

impl core::fmt::Display for FailedAllocation {
    /// Write a human-facing description of the error.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "memory allocation failed")
    }
}

impl core::error::Error for FailedAllocation {}

/// Error type for invalid index parameters.
#[derive(Debug, Clone, Copy)]
pub struct OutOfBounds;

impl core::fmt::Display for OutOfBounds {
    /// Write a human-facing description of the error.
    ///
    /// # Performance
    /// This methods takes O(1) time and consumes O(1) memory.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "index is outside the bounds of initialized elements")
    }
}

impl core::error::Error for OutOfBounds {}

#[cfg(test)]
mod test {
    use super::*;

    mod method {
        use super::*;

        mod with_capacity {
            use super::*;

            #[test]
            fn increases_capacity() {
                let actual = Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                assert_eq!(actual.capacity(), 256);
                assert_eq!(actual.capacity_front(), 256);
                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn allocates_memory() {
                let actual = Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                for index in 0..actual.capacity() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let actual = Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let _actual = Dynamic::<usize>::with_capacity(0).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let _actual =
                    Dynamic::<()>::with_capacity(capacity).expect("ZSTs do not occupy memory");
            }
        }

        mod capacity {
            use super::*;

            #[test]
            fn only_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity(), 256);
            }

            #[test]
            fn only_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity(), 256);
            }

            #[test]
            fn front_and_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");
                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity(), 512);
            }

            #[test]
            fn does_not_invalidate_pointers_for_that_many_additions() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                let ptr = actual.buffer.as_ptr();

                for index in 0..actual.capacity() {
                    if index % 2 == 0 {
                        _ = actual.append(index).expect("uses capacity");
                    } else {
                        _ = actual.prepend(index).expect("uses capacity");
                    }
                }

                assert_eq!(ptr, actual.buffer.as_ptr());
            }
        }

        mod capacity_front {
            use super::*;

            #[test]
            fn is_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity_front(), actual.front_capacity);
            }

            #[test]
            fn does_not_count_back_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn counts_back_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity_front(), 256);
            }

            #[test]
            fn does_not_invalidate_pointers_for_that_many_prepends() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                let ptr = actual.buffer.as_ptr();

                for index in 0..actual.capacity_front() {
                    _ = actual.prepend(index).expect("uses capacity");
                }

                assert_eq!(ptr, actual.buffer.as_ptr());
            }
        }

        mod capacity_back {
            use super::*;

            #[test]
            fn is_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity_back(), actual.back_capacity);
            }

            #[test]
            fn does_not_count_front_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn counts_front_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn does_not_invalidate_pointers_for_that_many_appends() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                let ptr = actual.buffer.as_ptr();

                for index in 0..actual.capacity_back() {
                    _ = actual.append(index).expect("uses capacity");
                }

                assert_eq!(ptr, actual.buffer.as_ptr());
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn increases_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve(1).expect("successful allocation");

                assert!(actual.capacity() >= 1);
            }

            #[test]
            fn increases_capacity_in_powers_of_two() {
                let mut actual = Dynamic::<()>::default();

                for _ in 0..(isize::BITS - 1) {
                    let capacity = actual.capacity() + 1;

                    _ = actual.reserve(capacity).expect("successful allocation");

                    let capacity = capacity
                        .checked_next_power_of_two()
                        .expect("loop conditions ensures this will fit");

                    assert_eq!(actual.capacity(), capacity);
                }
            }

            #[test]
            fn does_not_decrease_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.reserve(0).expect("does not alter allocation");

                assert_eq!(actual.capacity(), 256);
            }

            #[test]
            fn uses_front_capacity_before_reallocating() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                let existing_allocation = actual.buffer.as_ptr();

                _ = actual.reserve(256).expect("does no reallocation");

                assert_eq!(actual.buffer.as_ptr(), existing_allocation);
            }

            #[test]
            fn allocates_memory() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve(256).expect("successful allocation");

                for index in 0..actual.capacity() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .reserve(actual.capacity() * 2)
                    .expect("successful allocation");

                for index in 0..actual.capacity() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve(256).expect("successful allocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.reserve(256).expect("successful allocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve(0).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual = Dynamic::<()>::default();

                _ = actual.reserve(capacity).expect("ZSTs do not occupy memory");
            }
        }

        mod reserve_front {
            use super::*;

            #[test]
            fn increases_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity_front(), 256);
            }

            #[test]
            fn does_not_decrease_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.reserve_front(0).expect("does not alter allocation");

                assert_eq!(actual.capacity_front(), 256);
            }

            #[test]
            fn does_not_modify_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn allocates_memory() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_front(256).expect("successful allocation");

                for index in 0..actual.capacity_front() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .reserve_front(actual.capacity_front() * 2)
                    .expect("successful allocation");

                for index in 0..actual.capacity_front() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.reserve_front(256).expect("successful allocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_front(0).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual = Dynamic::<()>::default();

                _ = actual
                    .reserve_front(capacity)
                    .expect("ZSTs do not occupy memory");
            }
        }

        mod reserve_back {
            use super::*;

            #[test]
            fn increases_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn does_not_decrease_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.reserve_back(0).expect("does not alter allocation");

                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn does_not_modify_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.capacity_front(), 256);
            }

            #[test]
            fn allocates_memory() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_back(256).expect("successful allocation");

                for index in 0..actual.capacity_back() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .reserve_back(actual.capacity_back() * 2)
                    .expect("successful allocation");

                for index in 0..actual.capacity_back() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.reserve_back(256).expect("successful allocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_back(0).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual = Dynamic::<()>::default();

                _ = actual
                    .reserve_back(capacity)
                    .expect("ZSTs do not occupy memory");
            }
        }

        mod shrink {
            use super::*;

            #[test]
            fn decreases_capacity_when_some() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.shrink(Some(64)).expect("successful reallocation");

                assert_eq!(actual.capacity(), 64);
            }

            #[test]
            fn removes_capacity_when_none() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.shrink(None).expect("successful reallocation");

                assert_eq!(actual.capacity(), 0);
            }

            #[test]
            fn does_not_increase_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(64).expect("successful allocation");

                _ = actual.shrink(Some(256)).expect("does not alter allocation");

                assert_eq!(actual.capacity(), 64);
            }

            #[test]
            fn shrinks_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                _ = actual.shrink(None).expect("successful reallocation");

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn shrinks_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                _ = actual.shrink(None).expect("successful reallocation");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn shrinks_front_and_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");
                _ = actual.reserve_back(256).expect("successful allocation");

                _ = actual.shrink(None).expect("successful reallocation");

                assert_eq!(actual.capacity_front(), 0);
                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.shrink(Some(128)).expect("successful allocation");

                for index in 0..actual.capacity() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.shrink(Some(128)).expect("successful reallocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.shrink(None).expect("successful reallocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.shrink(None).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual =
                    Dynamic::<()>::with_capacity(capacity).expect("successful allocation");

                _ = actual.shrink(None).expect("ZSTs do not occupy memory");
            }
        }

        mod shrink_front {
            use super::*;

            #[test]
            fn decreases_front_capacity_when_some() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful reallocation");

                _ = actual
                    .shrink_front(Some(64))
                    .expect("successful reallocation");

                assert_eq!(actual.capacity_front(), 64);
            }

            #[test]
            fn removes_front_capacity_when_none() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful reallocation");

                _ = actual.shrink_front(None).expect("successful reallocation");

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn does_not_increase_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(64).expect("successful allocation");

                _ = actual
                    .shrink_front(Some(256))
                    .expect("does not alter allocation");

                assert_eq!(actual.capacity(), 64);
            }

            #[test]
            fn does_not_decrease_back_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful allocation");

                _ = actual.shrink_front(None).expect("no-op");

                assert_eq!(actual.capacity_back(), 256);
            }

            #[test]
            fn decreases_back_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_back(256).expect("successful allocation");

                _ = actual.shrink_front(None).expect("successful deallocation");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .shrink_front(Some(128))
                    .expect("successful allocation");

                for index in 0..actual.capacity_front() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .shrink_front(Some(128))
                    .expect("successful reallocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.shrink_front(None).expect("successful reallocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.shrink_front(None).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual =
                    Dynamic::<()>::with_capacity(capacity).expect("successful allocation");

                _ = actual
                    .shrink_front(None)
                    .expect("ZSTs do not occupy memory");
            }
        }

        mod shrink_back {
            use super::*;

            #[test]
            fn decreases_back_capacity_when_some() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful reallocation");

                _ = actual
                    .shrink_back(Some(64))
                    .expect("successful reallocation");

                assert_eq!(actual.capacity_back(), 64);
            }

            #[test]
            fn removes_back_capacity_when_none() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_back(256).expect("successful reallocation");

                _ = actual.shrink_back(None).expect("successful reallocation");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn does_not_increase_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(64).expect("successful allocation");

                _ = actual
                    .shrink_back(Some(256))
                    .expect("does not alter allocation");

                assert_eq!(actual.capacity(), 64);
            }

            #[test]
            fn does_not_decrease_front_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                _ = actual.shrink_back(None).expect("no-op");

                assert_eq!(actual.capacity_front(), 256);
            }

            #[test]
            fn decreases_front_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.reserve_front(256).expect("successful allocation");

                _ = actual.shrink_back(None).expect("successful deallocation");

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .shrink_back(Some(128))
                    .expect("successful allocation");

                for index in 0..actual.capacity_back() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_initialize_elements() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual
                    .shrink_back(Some(128))
                    .expect("successful reallocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.shrink_back(None).expect("successful reallocation");

                assert!(actual.eq(expected));
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.shrink_back(None).expect("does no allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let capacity = usize::try_from(isize::MAX).expect("usize::MAX > isize::MAX");

                let mut actual =
                    Dynamic::<()>::with_capacity(capacity).expect("successful allocation");

                _ = actual.shrink_back(None).expect("ZSTs do not occupy memory");
            }
        }

        mod shift {
            use super::*;

            #[test]
            fn left_increases_back_capacity_and_decreases_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
                _ = actual.reserve_front(256).expect("successful allocation");

                for _ in 0..256 {
                    let front_capacity = actual.front_capacity;
                    let back_capacity = actual.back_capacity;

                    _ = actual.shift(-1).expect("front capacity to shift into");

                    assert_eq!(actual.front_capacity, front_capacity - 1);
                    assert_eq!(actual.back_capacity, back_capacity + 1);
                }
            }

            #[test]
            fn left_does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual = Dynamic::from_iter(expected);
                _ = actual.reserve_front(256).expect("successful allocation");

                for _ in 0..256 {
                    _ = actual.shift(-1).expect("front capacity to shift into");

                    assert!(actual.iter().eq(expected.iter()));
                }
            }

            #[test]
            fn right_increases_front_capacity_and_decreases_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
                _ = actual.reserve_back(256).expect("successful allocation");

                for _ in 0..256 {
                    let front_capacity = actual.front_capacity;
                    let back_capacity = actual.back_capacity;

                    _ = actual.shift(1).expect("back capacity to shift into");

                    assert_eq!(actual.front_capacity, front_capacity + 1);
                    assert_eq!(actual.back_capacity, back_capacity - 1);
                }
            }

            #[test]
            fn right_does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual = Dynamic::from_iter(expected);
                _ = actual.reserve_back(256).expect("successful allocation");

                for _ in 0..256 {
                    _ = actual.shift(1).expect("right capacity to shift into");

                    assert!(actual.iter().eq(expected.iter()));
                }
            }

            #[test]
            fn zero_cannot_fail() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.shift(0).expect("does not alter allocation");
            }

            #[test]
            fn errors_when_out_of_bounds() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.shift(-1).expect_err("no front capacity");
                _ = actual.shift(1).expect_err("no back capacity");
            }

            #[test]
            fn when_empty() {
                let mut actual = Dynamic::<()>::default();

                _ = actual.shift(0).expect("does not alter allocation");
            }
        }

        mod remove_via_front {
            use super::*;

            #[test]
            fn yields_none_when_out_of_bounds() {
                let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                let actual = underlying.remove_via_front(underlying.len());

                assert_eq!(actual, None);
            }

            #[test]
            fn yields_element_when_in_bounds() {
                let underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                for index in 1..underlying.len() {
                    let mut underlying = underlying.clone();

                    let actual = underlying.remove_via_front(index);

                    assert_eq!(actual, Some(index));
                }
            }

            #[test]
            fn removed_becomes_first_element() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_front(3).expect("element with value '3'");

                assert_eq!(actual[2], 0);
            }

            #[test]
            fn does_not_modify_other_elements() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_front(1);

                assert!(actual.eq([0, 2, 3, 4, 5]));
            }

            #[test]
            fn increases_front_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_front(5);

                assert_eq!(actual.capacity_front(), 1);
            }

            #[test]
            fn when_front_element() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                let removed = actual.remove_via_front(0);

                assert_eq!(removed, Some(0));
                assert_eq!(actual.capacity_front(), 1);
                assert!(actual.eq([1, 2, 3, 4, 5]));
            }

            #[test]
            fn when_only_one_element() {
                let mut actual = Dynamic::from_iter([0]);

                let removed = actual.remove_via_front(0);

                assert_eq!(removed, Some(0));
                assert_eq!(actual.capacity_front(), 1);
                assert_eq!(actual.len(), 0);
            }
        }

        mod remove_via_back {
            use super::*;

            #[test]
            fn yields_none_when_out_of_bounds() {
                let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                let actual = underlying.remove_via_back(underlying.len());

                assert_eq!(actual, None);
            }

            #[test]
            fn yields_element_when_in_bounds() {
                let underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                for index in 1..underlying.len() {
                    let mut underlying = underlying.clone();

                    let actual = underlying.remove_via_back(index);

                    assert_eq!(actual, Some(index));
                }
            }

            #[test]
            fn removed_becomes_last_element() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_back(3).expect("element with value '3'");

                assert_eq!(actual[3], 5);
            }

            #[test]
            fn does_not_modify_other_elements() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_back(4);

                assert!(actual.eq([0, 1, 2, 3, 5]));
            }

            #[test]
            fn increases_back_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.remove_via_back(0);

                assert_eq!(actual.capacity_back(), 1);
            }

            #[test]
            fn when_back_element() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                let removed = actual.remove_via_back(5);

                assert_eq!(removed, Some(5));
                assert_eq!(actual.capacity_back(), 1);
                assert!(actual.eq([0, 1, 2, 3, 4]));
            }

            #[test]
            fn when_only_one_element() {
                let mut actual = Dynamic::from_iter([0]);

                let removed = actual.remove_via_back(0);

                assert_eq!(removed, Some(0));
                assert_eq!(actual.capacity_back(), 1);
                assert_eq!(actual.len(), 0);
            }
        }

        mod resize {
            use super::*;

            #[test]
            fn does_not_initialize_elements() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.resize(256).expect("successful allocation");

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn increases_back_capacity() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.resize(256).expect("successful allocation");

                assert_eq!(actual.back_capacity, 256);
            }

            #[test]
            fn does_not_increase_front_capacity() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.resize(256).expect("successful allocation");

                assert_eq!(actual.front_capacity, 0);
            }

            #[test]
            fn decreases_back_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.resize(-128).expect("successful allocation");

                assert_eq!(actual.back_capacity, 128);
            }

            #[test]
            fn does_not_decrease_front_capacity() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.resize(-128).expect("successful allocation");

                assert_eq!(actual.front_capacity, 0);
            }

            #[test]
            fn errors_when_input_would_drop_initialized_elements() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                for elements in 1..=actual.initialized {
                    let elements =
                        isize::try_from(elements).expect("there are less than isize::MAX elements");

                    _ = actual.resize(-elements).expect_err("not enough capacity");
                }
            }

            #[test]
            fn allocates_memory() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.resize(256).expect("successful allocation");

                for index in 0..actual.capacity_back() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn reallocates_memory() {
                let mut actual =
                    Dynamic::<usize>::with_capacity(256).expect("successful allocation");

                _ = actual.resize(-128).expect("successful reallocation");

                for index in 0..actual.capacity_back() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn does_not_modify_initialized_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.resize(128).expect("successful reallocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn zero_capacity_cannot_fail() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.resize(0).expect("does not alter allocation");
            }

            #[test]
            fn zero_size_types_cannot_fail() {
                let mut actual = Dynamic::<()>::default();

                _ = actual
                    .resize(isize::MAX)
                    .expect("ZSTs do not occupy memory");
                _ = actual
                    .resize(-isize::MAX)
                    .expect("ZSTs do not occupy memory");
            }
        }
    }

    mod drop {
        use super::*;

        #[test]
        fn zero_size_type() {
            drop(Dynamic::<()>::default());
        }

        #[test]
        fn empty() {
            drop(Dynamic::<usize>::default());
        }

        #[test]
        fn all_initialized() {
            use crate::test::mock::DropCounter;

            const ELEMENTS: usize = 256;

            let dropped = DropCounter::new_counter();

            let mut actual =
                Dynamic::<DropCounter>::with_capacity(ELEMENTS).expect("successful allocation");

            for _ in 0..ELEMENTS {
                _ = actual
                    .append(DropCounter::new(&dropped))
                    .expect("uses capacity");
            }

            drop(actual);

            assert_eq!(dropped.take(), ELEMENTS);
        }

        #[test]
        fn all_front_capacity() {
            let mut actual = Dynamic::<usize>::default();

            _ = actual.reserve_front(256).expect("successful allocation");

            drop(actual);
        }

        #[test]
        fn all_back_capacity() {
            let mut actual = Dynamic::<usize>::default();

            _ = actual.reserve_back(256).expect("successful allocation");

            drop(actual);
        }

        #[test]
        fn front_capacity_and_initialized_elements_and_back_capacity() {
            use crate::test::mock::DropCounter;

            const ELEMENTS: usize = 256;

            let dropped = DropCounter::new_counter();

            let mut actual =
                Dynamic::<DropCounter>::with_capacity(ELEMENTS).expect("successful allocation");

            for _ in 0..ELEMENTS {
                _ = actual
                    .append(DropCounter::new(&dropped))
                    .expect("uses capacity");
            }

            _ = actual.reserve_front(256).expect("successful allocation");
            _ = actual.reserve_back(256).expect("successful allocation");

            drop(actual);

            assert_eq!(dropped.take(), ELEMENTS);
        }
    }

    mod try_from {
        use super::*;

        #[test]
        fn does_not_allocate_front_capacity() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");

            assert_eq!(actual.front_capacity, 0);
        }

        #[test]
        fn does_not_allocate_back_capacity() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");

            assert_eq!(actual.back_capacity, 0);
        }

        #[test]
        fn allocates_memory() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");

            for index in 0..expected.len() {
                let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                // Ideally, this will seg-fault if unowned memory.
                _ = unsafe { &mut *ptr }.write(index);
            }
        }

        #[test]
        fn has_elements() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");

            assert_eq!(actual.initialized, expected.len());
        }

        #[test]
        fn initializes_elements() {
            let expected = [0, 1, 2, 3, 4, 5];

            let actual = Dynamic::try_from(expected.as_slice()).expect("successful allocation");

            for index in 0..expected.len() {
                assert_eq!(actual[index], expected[index]);
            }
        }
    }

    mod index {
        use super::*;

        use core::ops::Index as _;

        #[test]
        fn correct_element() {
            let expected = [0, 1, 2, 3, 4, 5];
            let actual = Dynamic::from_iter(expected);

            for (index, value) in expected.iter().enumerate() {
                assert_eq!(actual.index(index), value);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let instance = Dynamic::<()>::default();

            let _: &() = instance.index(0);
        }
    }

    mod index_mut {
        use super::*;

        use core::ops::IndexMut as _;

        #[test]
        fn correct_element() {
            let mut expected = [0, 1, 2, 3, 4, 5];
            let mut actual = Dynamic::from_iter(expected);

            for (index, value) in expected.iter_mut().enumerate() {
                assert_eq!(actual.index_mut(index), value);
            }
        }

        #[test]
        #[should_panic = "index out of bounds"]
        fn panics_when_out_of_bounds() {
            let mut instance = Dynamic::<()>::default();

            let _: &mut () = instance.index_mut(0);
        }

        #[test]
        fn is_mutable() {
            let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

            for element in actual.iter_mut() {
                *element = 0;
            }

            for element in actual {
                assert_eq!(element, 0);
            }
        }
    }

    mod iterator {
        use super::*;

        mod into {
            use super::*;

            #[test]
            fn element_count() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert_eq!(actual.into_iter().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert!(actual.into_iter().eq(expected.into_iter()));
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                                Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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

                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.into_iter().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let expected = [0, 1, 2, 3, 4, 5];

                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert!(actual.into_iter().rev().eq(expected.into_iter().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(
                        actual.into_iter().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.into_iter().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let mut actual: Dynamic<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();

                    for remaining in (0..actual.len()).rev() {
                        _ = actual.next();

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let actual = Dynamic::<()>::default();
                    let mut actual = actual.into_iter();

                    // Yields `None` at least once.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);

                    // Continues to yield `None`.
                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn exhausted() {
                    let actual: Dynamic<_> = [()].into_iter().collect();
                    let mut actual = actual.into_iter();

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
            fn does_not_allocate_front_capacity() {
                let actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                assert_eq!(actual.front_capacity, 0);
            }

            #[test]
            fn does_not_allocate_back_capacity() {
                let actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                assert_eq!(actual.back_capacity, 0);
            }

            #[test]
            fn allocates_memory() {
                let expected = [0, 1, 2, 3, 4, 5];
                let actual: Dynamic<_> = expected.iter().copied().collect();

                for index in 0..expected.len() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn updates_internal_state() {
                let expected = [0, 1, 2, 3, 4, 5];
                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert_eq!(actual.initialized, expected.len());
            }

            #[test]
            fn initializes_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let actual: Dynamic<_> = expected.iter().copied().collect();

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn empty() {
                let actual: Dynamic<()> = core::iter::empty().collect();

                assert_eq!(actual.front_capacity, 0);
                assert_eq!(actual.initialized, 0);
                assert_eq!(actual.back_capacity, 0);
            }

            #[test]
            fn handles_oversized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let actual: Dynamic<_> = SizeHint {
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

                let actual: Dynamic<_> = SizeHint {
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

                let actual: Dynamic<_> = SizeHint {
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

                let actual: Dynamic<_> = SizeHint {
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
            fn does_not_allocate_front_capacity() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];
                actual.extend(expected);

                assert_eq!(actual.front_capacity, 0);
            }

            #[test]
            fn does_not_allocate_back_capacity() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];
                actual.extend(expected);

                assert_eq!(actual.back_capacity, 0);
            }

            #[test]
            fn consumes_front_capacity() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];

                _ = actual
                    .reserve_front(expected.len())
                    .expect("successful allocation");

                actual.extend(expected);

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn consumes_back_capacity() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];

                _ = actual
                    .reserve_back(expected.len())
                    .expect("successful allocation");

                actual.extend(expected);

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn allocates_memory_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];
                actual.extend(expected);

                for index in 0..expected.len() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn reallocates_memory_when_not_enough_capacity() {
                let mut actual = Dynamic::<usize>::with_capacity(1).expect("successful allocation");

                let expected = [0, 1, 2, 3, 4, 5];
                actual.extend(expected);

                for index in 0..expected.len() {
                    let ptr = unsafe { actual.buffer.as_ptr().add(index) };

                    // Ideally, this will seg-fault if unowned memory.
                    _ = unsafe { &mut *ptr }.write(index);
                }
            }

            #[test]
            fn updates_internal_state() {
                let mut actual = Dynamic::default();

                let expected = [0, 1, 2, 3, 4, 5];

                actual.extend(expected);

                assert_eq!(actual.initialized, expected.len());
            }

            #[test]
            fn appends_elements() {
                let preexisting = [0, 1, 2];
                let mut actual: Dynamic<_> = preexisting.into_iter().collect();

                let expected = [3, 4, 5];
                actual.extend(expected.iter().copied());

                #[expect(clippy::shadow_unrelated, reason = "elements from them")]
                for (actual, expected) in actual.skip(preexisting.len()).zip(expected) {
                    assert_eq!(actual, expected);
                }
            }

            #[test]
            fn does_not_modify_other_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                actual.extend([6, 7, 8, 9, 10]);

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn into_empty_instance() {
                let mut actual = Dynamic::<usize>::default();

                let expected = [0, 1, 2, 3, 4, 5];

                actual.extend(expected.iter().copied());

                assert!(actual.eq(expected));
            }

            #[test]
            fn from_empty_iterator() {
                let mut actual = Dynamic::<()>::default();

                actual.extend(core::iter::empty());

                assert_eq!(actual.front_capacity, 0);
                assert_eq!(actual.initialized, 0);
                assert_eq!(actual.back_capacity, 0);
            }

            #[test]
            fn handles_oversized_size_hint() {
                use crate::test::mock::SizeHint;

                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual = Dynamic::default();

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

                let mut actual = Dynamic::default();

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

                let mut actual = Dynamic::default();

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

                let mut actual = Dynamic::default();

                actual.extend(SizeHint {
                    data: expected.iter().copied(),
                    size_hint: (0, None),
                });

                assert_eq!(actual.len(), expected.len());
            }
        }
    }

    mod default {
        use super::*;

        #[test]
        fn does_not_allocate_front_capacity() {
            let actual = Dynamic::<usize>::default();

            assert_eq!(actual.front_capacity, 0);
        }

        #[test]
        fn does_not_allocate_back_capacity() {
            let actual = Dynamic::<usize>::default();

            assert_eq!(actual.back_capacity, 0);
        }

        #[test]
        fn does_not_initialize_elements() {
            let actual = Dynamic::<()>::default();

            assert_eq!(actual.initialized, 0);
        }
    }

    mod clone {
        use super::*;

        #[test]
        fn does_not_allocate_front_capacity() {
            let actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]).clone().clone();

            assert_eq!(actual.front_capacity, 0);
        }

        #[test]
        fn does_not_allocate_back_capacity() {
            let actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]).clone().clone();

            assert_eq!(actual.back_capacity, 0);
        }

        #[test]
        fn has_elements() {
            let expected = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

            let actual = expected.clone();

            assert_eq!(actual.initialized, expected.len());
        }

        #[test]
        fn is_equivalent() {
            let expected = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

            let actual = expected.clone();

            assert_eq!(actual, expected);
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn eq_when_same_elements() {
            let expected = [0, 1, 2, 3, 4, 5];

            let first: Dynamic<_> = expected.iter().copied().collect();
            let second: Dynamic<_> = expected.iter().copied().collect();

            assert_eq!(first, second);
        }

        #[test]
        fn ne_when_different_elements() {
            let first = Dynamic::from_iter([0]);
            let second = Dynamic::from_iter([1]);

            assert_ne!(first, second);
        }

        #[test]
        fn ignores_different_front_capacity() {
            let expected = [0, 1, 2, 3, 4, 5];

            let mut first: Dynamic<_> = expected.iter().copied().collect();
            let mut second: Dynamic<_> = expected.iter().copied().collect();

            _ = first.reserve_front(128).expect("successful allocation");
            _ = second.reserve_front(256).expect("successful allocation");

            assert_eq!(first, second);
        }

        #[test]
        fn ignores_different_back_capacity() {
            let expected = [0, 1, 2, 3, 4, 5];

            let mut first: Dynamic<_> = expected.iter().copied().collect();
            let mut second: Dynamic<_> = expected.iter().copied().collect();

            _ = first.reserve_back(128).expect("successful allocation");
            _ = second.reserve_back(256).expect("successful allocation");

            assert_eq!(first, second);
        }

        #[test]
        fn is_symmetric() {
            let expected = [0, 1, 2, 3, 4, 5];

            let first: Dynamic<_> = expected.iter().copied().collect();
            let second: Dynamic<_> = expected.iter().copied().collect();

            // `first == second` <=> `second == first`
            assert_eq!(first, second);
            assert_eq!(second, first);
        }

        #[test]
        fn is_transitive() {
            let expected = [0, 1, 2, 3, 4, 5];

            let first: Dynamic<_> = expected.iter().copied().collect();
            let second: Dynamic<_> = expected.iter().copied().collect();
            let third: Dynamic<_> = expected.iter().copied().collect();

            // `first == second && second == third` => `first == third`
            assert_eq!(first, second);
            assert_eq!(second, third);
            assert_eq!(third, first);
        }

        #[test]
        fn is_reflexive() {
            let actual = Dynamic::<()>::default();

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
                let actual: Dynamic<_> = expected.iter().copied().collect();

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
                let expected = [0, 1, 2, 3, 4, 5];
                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert_eq!(Collection::count(&actual), expected.len());
            }

            #[test]
            fn zero_when_empty() {
                let actual = Dynamic::<()>::default();

                assert_eq!(Collection::count(&actual), 0);
            }

            #[test]
            fn ignores_front_capacity() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.count(), expected.len());
            }

            #[test]
            fn ignores_back_capacity() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.reserve_back(256).expect("successful allocation");

                assert_eq!(actual.count(), expected.len());
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
                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert_eq!(actual.iter().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let expected = [0, 1, 2, 3, 4, 5];
                let actual: Dynamic<_> = expected.iter().copied().collect();

                assert!(actual.iter().eq(expected.iter()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert!(actual.iter().rev().eq(expected.iter().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(
                        actual.iter().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let actual: Dynamic<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();
                    let mut actual = actual.iter();

                    for remaining in (0..actual.len()).rev() {
                        _ = actual.next();

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let actual = Dynamic::<()>::default();
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
                    let actual: Dynamic<_> = [()].into_iter().collect();
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
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                assert_eq!(actual.iter_mut().count(), expected.len());
            }

            #[test]
            fn in_order() {
                let mut expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                assert!(actual.iter_mut().eq(expected.iter_mut()));
            }

            mod double_ended {
                use super::*;

                #[test]
                fn element_count() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter_mut().rev().count(), expected.len());
                }

                #[test]
                fn in_order() {
                    let mut expected = [0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert!(actual.iter_mut().rev().eq(expected.iter_mut().rev()));
                }
            }

            mod exact_size {
                use super::*;

                #[test]
                fn hint() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(
                        actual.iter_mut().size_hint(),
                        (expected.len(), Some(expected.len()))
                    );
                }

                #[test]
                fn len() {
                    let expected = [0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.iter_mut().len(), expected.len());
                }

                #[test]
                fn updates() {
                    let mut actual: Dynamic<_> = [0, 1, 2, 3, 4, 5].into_iter().collect();
                    let mut actual = actual.iter_mut();

                    for remaining in (0..actual.len()).rev() {
                        _ = actual.next();

                        assert_eq!(actual.len(), remaining);
                    }
                }
            }

            mod fused {
                use super::*;

                #[test]
                fn empty() {
                    let mut actual = Dynamic::<()>::default();
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
                    let mut actual: Dynamic<_> = [()].into_iter().collect();
                    let mut actual = actual.iter_mut();

                    // Exhaust the elements.
                    let _: &mut () = actual.next().expect("the one element");

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
            fn address_of_underlying_buffer() {
                let actual = Dynamic::<i32>::from_iter([0, 1, 2, 3, 4, 5]);

                assert_eq!(
                    actual.as_ptr(),
                    actual.buffer.as_ptr().cast::<i32>().cast_const()
                );
            }

            #[test]
            fn skips_front_capacity() {
                let mut actual = Dynamic::<i32>::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.as_ptr(), unsafe {
                    actual.buffer.as_ptr().cast::<i32>().cast_const().add(256)
                });
            }

            #[test]
            #[should_panic = "no allocation to point to"]
            fn panics_if_no_allocation() {
                let actual = Dynamic::<()>::default();

                _ = actual.as_ptr();
            }
        }

        mod as_mut_ptr {
            use super::*;

            #[test]
            fn address_of_underlying_buffer() {
                let mut actual = Dynamic::<i32>::from_iter([0, 1, 2, 3, 4, 5]);

                assert_eq!(actual.as_mut_ptr(), actual.buffer.as_ptr().cast::<i32>());
            }

            #[test]
            fn skips_front_capacity() {
                let mut actual = Dynamic::<i32>::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.reserve_front(256).expect("successful allocation");

                assert_eq!(actual.as_mut_ptr(), unsafe {
                    actual.buffer.as_ptr().cast::<i32>().add(256)
                });
            }

            #[test]
            #[should_panic = "no allocation to point to"]
            fn panics_if_no_allocation() {
                let mut actual = Dynamic::<()>::default();

                _ = actual.as_mut_ptr();
            }
        }
    }

    mod list {
        use super::*;

        use crate::structure::collection::linear::list::List as _;

        mod insert {
            use super::*;

            #[test]
            fn adds_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual.initialized, expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual[2], 12345);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.insert(2, 12345).expect("successful allocation");

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.insert(2, 12345).expect("successful allocation");

                *actual = 54321;

                assert_eq!(actual, &mut 54321);
            }

            #[test]
            fn will_allocate_when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.insert(0, 12345).expect("successful allocation");
            }

            #[test]
            fn will_reallocate_when_no_capacity() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();
                _ = actual.shrink(None).expect("no capacity");

                _ = actual.insert(2, 12345).expect("successful allocation");
            }

            #[test]
            fn does_not_modify_leading_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.insert(INDEX, 12345).expect("successful allocation");

                for index in 0..INDEX {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.insert(INDEX, 12345).expect("successful allocation");

                for index in INDEX..expected.len() {
                    assert_eq!(actual[index + 1], expected[index]);
                }
            }

            #[test]
            fn prepending_reallocates_when_no_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.insert(0, 12345).expect("successful allocation");
            }

            #[test]
            fn prepending_consumes_front_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
                _ = actual.reserve_front(1).expect("successful allocation");

                _ = actual.insert(0, 12345).expect("uses front capacity");

                assert_eq!(actual.capacity_front(), 0);
            }

            #[test]
            fn prepending_consumes_back_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();
                _ = actual.reserve_back(1).expect("successful allocation");

                _ = actual.insert(0, 12345).expect("uses back capacity");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn appending_reallocates_when_no_capacity() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                _ = actual.insert(6, 12345).expect("successful allocation");
            }

            #[test]
            fn appending_consumes_back_capacity_when_not_empty() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);
                _ = actual.reserve_back(1).expect("successful allocation");

                _ = actual.insert(6, 12345).expect("uses back capacity");

                assert_eq!(actual.capacity_back(), 0);
            }

            #[test]
            fn appending_consumes_front_capacity_when_empty() {
                let mut actual = Dynamic::<usize>::default();
                _ = actual.reserve_front(1).expect("successful allocation");

                _ = actual.insert(0, 12345).expect("uses front capacity");

                assert_eq!(actual.capacity_front(), 0);
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.remove(0);

                assert_eq!(actual.initialized, expected.len() - 1);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                (0..expected.len()).for_each(|index| {
                    assert_eq!(actual.remove(0).expect("front element"), expected[index]);
                });
            }

            #[test]
            fn does_not_modify_leading_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.remove(INDEX);

                for index in 0..INDEX {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                const INDEX: usize = 2;

                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.remove(INDEX);

                for index in INDEX..expected.len() - 1 {
                    assert_eq!(actual[index], expected[index + 1]);
                }
            }

            #[test]
            fn none_when_index_out_of_bounds() {
                let mut actual = Dynamic::<()>::default();

                assert!(actual.remove(0).is_none());
            }

            #[test]
            fn increases_front_capacity_if_first_element() {
                let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                for index in 0..actual.len() {
                    _ = actual.remove(0).expect("element to remove");

                    assert_eq!(actual.capacity_front(), index + 1);
                }
            }
        }

        mod drain {
            use super::*;

            mod iterator {
                use super::*;

                #[test]
                fn yields_no_elements_when_empty() {
                    let mut actual = Dynamic::<()>::default();

                    let mut actual = actual.drain(..);

                    assert_eq!(actual.next(), None);

                    drop(actual);
                }

                #[test]
                fn yields_no_elements_when_start_of_range_is_out_of_bounds() {
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    let mut actual = actual.drain(6..);

                    assert_eq!(actual.next(), None);
                    assert_eq!(actual.next_back(), None);
                }

                #[test]
                fn yields_elements_when_end_of_range_is_out_of_bounds() {
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = actual.drain(..usize::MAX);

                    assert!(actual.eq([0, 1, 2, 3, 4, 5]));
                }

                #[test]
                fn element_count() {
                    let mut expected = vec![0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert_eq!(actual.drain(1..4).count(), expected.drain(1..4).count());
                }

                #[test]
                fn in_order() {
                    let mut expected = vec![0, 1, 2, 3, 4, 5];
                    let mut actual: Dynamic<_> = expected.iter().copied().collect();

                    assert!(actual.drain(1..4).eq(expected.drain(1..4)));
                }

                mod double_ended {
                    use super::*;

                    #[test]
                    fn yields_no_elements_when_empty() {
                        let mut actual = Dynamic::<()>::default();

                        let mut actual = actual.drain(..);

                        assert_eq!(actual.next_back(), None);

                        drop(actual);
                    }

                    #[test]
                    fn yields_no_elements_when_start_of_range_is_out_of_bounds() {
                        let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        let mut actual = actual.drain(6..);

                        assert_eq!(actual.next_back(), None);
                    }

                    #[test]
                    fn yields_elements_when_end_of_range_is_out_of_bounds() {
                        let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = actual.drain(..usize::MAX).rev();

                        assert!(actual.eq([5, 4, 3, 2, 1, 0]));
                    }

                    #[test]
                    fn element_count() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        assert_eq!(
                            actual.drain(1..4).rev().count(),
                            expected.drain(1..4).rev().count()
                        );
                    }

                    #[test]
                    fn in_order() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        assert!(actual.drain(1..4).rev().eq(expected.drain(1..4).rev()));
                    }

                    #[test]
                    fn prevents_elements_from_being_yielded_more_than_once() {
                        let mut underlying = Dynamic::from_iter([0, 1, 2, 0]);

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
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        let expected = expected.drain(1..4);

                        assert_eq!(
                            actual.drain(1..4).size_hint(),
                            (expected.len(), Some(expected.len()))
                        );
                    }

                    #[test]
                    fn len() {
                        let mut expected = vec![0, 1, 2, 3, 4, 5];
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        assert_eq!(actual.drain(1..4).len(), expected.drain(1..4).len());
                    }
                }

                mod fused {
                    use super::*;

                    #[test]
                    fn when_empty() {
                        let mut actual = Dynamic::<()>::default();
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
                        let mut actual: Dynamic<_> = [()].into_iter().collect();
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                                Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.drain(..));

                    assert_eq!(actual.initialized, 0);
                }

                #[test]
                fn does_not_modify_leading_elements() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 0..ELEMENTS {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        drop(actual.drain(start..));

                        assert!(actual.iter().eq(expected[..start].iter()));
                    }
                }

                #[test]
                fn does_not_modify_trailing_elements() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for end in 0..ELEMENTS {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

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
                            let mut actual: Dynamic<_> = expected.iter().copied().collect();

                            drop(actual.drain(start..end));

                            let expected_leading = expected[..start].iter();
                            let expected_trailing = expected[end..].iter();

                            assert!(actual.iter().eq(expected_leading.chain(expected_trailing)));
                        }
                    }
                }

                #[test]
                fn increases_front_capacity_when_range_starts_at_front_but_ends_before_back() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for end in 0..ELEMENTS {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();
                        debug_assert_eq!(actual.front_capacity, 0);

                        drop(actual.drain(..end));

                        assert_eq!(actual.front_capacity, end);
                    }
                }

                #[test]
                fn increases_back_capacity_when_range_ends_at_back_but_starts_before_front() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 0..ELEMENTS {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();
                        debug_assert_eq!(actual.back_capacity, 0);

                        drop(actual.drain(start..));

                        assert_eq!(actual.back_capacity, ELEMENTS - start);
                    }
                }

                #[test]
                fn increases_front_capacity_if_that_is_only_capacity() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 1..(ELEMENTS - 1) {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        // Add front capacity.
                        _ = actual
                            .reserve_front(ELEMENTS)
                            .expect("successful allocation");
                        debug_assert_eq!(actual.back_capacity, 0);

                        let removed = start..(ELEMENTS - 1);

                        drop(actual.drain(removed.clone()));

                        assert_eq!(actual.front_capacity, ELEMENTS + removed.len());
                    }
                }

                #[test]
                fn increases_back_capacity_if_that_is_only_capacity() {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 1..(ELEMENTS - 1) {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        // Add front capacity.
                        _ = actual
                            .reserve_back(ELEMENTS)
                            .expect("successful allocation");
                        debug_assert_eq!(actual.front_capacity, 0);

                        let removed = start..(ELEMENTS - 1);

                        drop(actual.drain(removed.clone()));

                        assert_eq!(actual.back_capacity, ELEMENTS + removed.len());
                    }
                }

                #[test]
                fn increases_front_capacity_when_front_and_back_capacity_but_more_trailing_elements()
                 {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in 1..(ELEMENTS / 2) {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        _ = actual
                            .reserve_front(ELEMENTS)
                            .expect("successful allocation");
                        _ = actual
                            .reserve_back(ELEMENTS)
                            .expect("successful allocation");

                        let removed = start..(ELEMENTS / 2);

                        drop(actual.drain(removed.clone()));

                        assert_eq!(actual.front_capacity, ELEMENTS + removed.len());
                    }
                }

                #[test]
                fn increases_back_capacity_when_front_and_back_capacity_but_more_leading_elements()
                {
                    const ELEMENTS: usize = 8;

                    let expected = core::array::from_fn::<_, ELEMENTS, _>(|index| index);

                    for start in (ELEMENTS / 2)..ELEMENTS {
                        let mut actual: Dynamic<_> = expected.iter().copied().collect();

                        _ = actual
                            .reserve_front(ELEMENTS)
                            .expect("successful allocation");
                        _ = actual
                            .reserve_back(ELEMENTS)
                            .expect("successful allocation");

                        let removed = start..(ELEMENTS / 2);

                        drop(actual.drain(removed.clone()));

                        assert_eq!(actual.back_capacity, ELEMENTS + removed.len());
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
                    let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = underlying.withdraw(|element| element % 2 == 0);

                    assert_eq!(actual.count(), 3);
                }

                #[test]
                fn in_order() {
                    let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = underlying.withdraw(|element| element % 2 == 0);

                    assert!(actual.eq([0, 2, 4]));
                }

                #[test]
                fn size_hint() {
                    let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    let actual = underlying.withdraw(|element| element % 2 == 0);

                    assert_eq!(actual.size_hint(), (0, Some(6)));
                }

                mod double_ended {
                    use super::*;

                    #[test]
                    fn element_count() {
                        let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = underlying.withdraw(|element| element % 2 == 0).rev();

                        assert_eq!(actual.count(), 3);
                    }

                    #[test]
                    fn in_order() {
                        let mut underlying = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        let actual = underlying.withdraw(|element| element % 2 == 0).rev();

                        assert!(actual.eq([4, 2, 0]));
                    }

                    #[test]
                    fn increases_back_capacity_when_withdrawing_last_element() {
                        let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        drop(actual.withdraw(|element| element > &0).rev());

                        assert_eq!(actual.capacity_front(), 0);
                        assert_eq!(actual.capacity_back(), 5);
                    }

                    #[test]
                    fn increases_back_capacity_when_retained_are_combined() {
                        let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        drop(actual.withdraw(|element| element % 2 == 1).rev());

                        assert_eq!(actual.capacity_front(), 0);
                        assert_eq!(actual.capacity_back(), 3);
                    }

                    #[test]
                    fn combines_retained_elements() {
                        let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                        drop(actual.withdraw(|element| element == &1).rev());

                        assert!(actual.eq([0, 2, 3, 4, 5]));
                    }

                    #[test]
                    fn prevents_elements_from_being_yielded_more_than_once() {
                        let mut underlying = Dynamic::from_iter([0, 1, 2, 0]);

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
                        let mut underlying = Dynamic::<usize>::default();
                        let mut actual = underlying.withdraw(|element| element % 2 == 0);

                        // Yields `None` at least once.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);

                        // Continues to yield `None`.
                        assert_eq!(actual.next(), None);
                        assert_eq!(actual.next_back(), None);
                    }

                    #[test]
                    fn exhausted() {
                        let mut underlying = Dynamic::from_iter([0]);
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                                Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|_| {
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
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.withdraw(|_| true));

                    assert_eq!(actual.initialized, 0);
                }

                #[test]
                fn does_not_modify_retained_elements() {
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5]);

                    drop(actual.withdraw(|element| element % 2 == 0));

                    assert!(actual.eq([1, 3, 5]));
                }

                #[test]
                fn increases_capacity_by_amount_of_removed_elements() {
                    const ELEMENTS: usize = 8;

                    for count in 0..=ELEMENTS {
                        let mut actual =
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|index| {
                                index
                            }));
                        debug_assert_eq!(actual.capacity(), 0);

                        drop(actual.withdraw(|element| element < &count));

                        assert_eq!(actual.capacity(), count);
                    }
                }

                #[test]
                fn first_retained_element_is_not_repositioned() {
                    const ELEMENTS: usize = 8;

                    for retained in 0..ELEMENTS {
                        let mut actual =
                            Dynamic::from_iter(core::array::from_fn::<_, ELEMENTS, _>(|index| {
                                index
                            }));
                        debug_assert_eq!(actual.front_capacity, 0);
                        debug_assert_eq!(actual.back_capacity, 0);

                        drop(actual.withdraw(|element| element != &retained));

                        assert_eq!(actual.front_capacity, retained);
                        assert_eq!(actual.back_capacity, ELEMENTS - retained - 1);
                    }
                }

                #[test]
                fn combines_disjointed_retained_elements_after_exhaustion() {
                    let mut actual = Dynamic::from_iter([0, 1, 2, 3, 4, 5, 6, 7]);

                    let mut iter = actual.withdraw(|element| element == &3 || element == &4);

                    // Create two disjointed regions of retained elements.
                    // The first region contains `[0, 1, 2`.
                    // The second regions contains `[5, 6, 7]`.
                    // The following calls will remove elements 3 and 4,
                    // alongside exhausting both sides of the iterator.
                    // However, we know from the implementation that neither
                    // call will move any retained elements keeping the gap.
                    _ = iter.next_back().expect("the element with value '4'");
                    _ = iter.next().expect("the element with value '3'");

                    // The above means it is now the responsibility of `drop`
                    // to combine these two regions thereby fixing the state of
                    // the underlying buffer for future use.
                    drop(iter);

                    assert!(actual.eq([0, 1, 2, 5, 6, 7]));
                }
            }
        }

        mod clear {
            use super::*;

            #[test]
            fn drops_all_elements() {
                use crate::test::mock::DropCounter;

                const ELEMENTS: usize = 256;

                let dropped = DropCounter::new_counter();

                let mut actual =
                    Dynamic::<DropCounter>::with_capacity(ELEMENTS).expect("successful allocation");

                for _ in 0..ELEMENTS {
                    _ = actual
                        .append(DropCounter::new(&dropped))
                        .expect("uses capacity");
                }

                actual.clear();

                assert_eq!(actual.initialized, 0);
                assert_eq!(dropped.take(), ELEMENTS);
            }

            #[test]
            fn keeps_allocation() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual = Dynamic::from_iter(expected);

                actual.clear();

                assert_eq!(actual.capacity(), expected.len());
            }

            #[test]
            fn when_already_empty() {
                let mut actual = Dynamic::<usize>::default();

                // Ideally this will panic or something in case of logic error.
                actual.clear();
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
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Dynamic<_> = [1, 2, 3, 4, 5].into_iter().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual[0], 0);
            }

            #[test]
            fn yields_element() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.push(0).expect("successful allocation");

                assert_eq!(actual, &mut 0);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.push(0).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index + 1], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.push(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod pop {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.pop();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for offset in 1..=expected.len() {
                    _ = actual.pop();

                    assert!(actual.iter().eq(expected[offset..].iter()));
                }

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.pop(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Dynamic::<()>::default();

                assert_eq!(actual.pop(), None);
            }
        }

        mod peek {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.peek(), Some(&element));

                    _ = actual.pop();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Dynamic::<()>::default();

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
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.push(0).expect("successful allocation");

                assert_eq!(actual.len(), expected.len() + 1);
            }

            #[test]
            fn initializes_element() {
                let mut actual: Dynamic<_> = [0, 1, 2, 3, 4].into_iter().collect();

                _ = actual.push(5).expect("successful allocation");

                assert_eq!(actual[5], 5);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.push(5).expect("successful allocation");

                assert_eq!(actual, &mut 5);
            }

            #[test]
            fn returned_reference_is_mutable() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                let actual = actual.push(5).expect("successful allocation");

                *actual = 12345;

                assert_eq!(actual, &mut 12345);
            }

            #[test]
            fn does_not_modify_leading_elements() {
                let expected = [0, 1, 2, 3, 4];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                _ = actual.push(5).expect("successful allocation");

                for index in 0..expected.len() {
                    assert_eq!(actual[index], expected[index]);
                }
            }

            #[test]
            fn when_empty() {
                let mut actual = Dynamic::<usize>::default();

                _ = actual.push(0).expect("successful allocation");

                assert!(actual.eq([0]));
            }
        }

        mod pop {
            use super::*;

            #[test]
            fn subtracts_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for remaining in (0..expected.len()).rev() {
                    _ = actual.pop();

                    assert_eq!(actual.len(), remaining);
                }
            }

            #[test]
            fn does_not_modify_trailing_elements() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for offset in 1..=expected.len() {
                    _ = actual.pop();

                    assert!(actual.iter().eq(expected[offset..].iter()));
                }

                assert_eq!(actual.initialized, 0);
            }

            #[test]
            fn yields_element() {
                let expected = [0, 1, 2, 3, 4, 5];
                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.pop(), Some(element));
                }
            }

            #[test]
            fn none_when_empty() {
                let mut actual = Dynamic::<()>::default();

                assert_eq!(actual.pop(), None);
            }
        }

        mod peek {
            use super::*;

            #[test]
            fn correct_element() {
                let expected = [0, 1, 2, 3, 4, 5];

                let mut actual: Dynamic<_> = expected.iter().copied().collect();

                for element in expected {
                    assert_eq!(actual.peek(), Some(&element));

                    _ = actual.pop();
                }
            }

            #[test]
            fn none_when_empty() {
                let actual = Dynamic::<()>::default();

                assert_eq!(actual.peek(), None);
            }
        }
    }
}

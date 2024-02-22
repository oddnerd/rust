//! Implementation of [`Fixed`].

use super::Array;
use super::Collection;
use super::Linear;

pub mod iter;
pub mod ops;
/// Fixed size (statically stack allocated) [`Array`].
pub struct Fixed<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> std::convert::From<[T; N]> for Fixed<T, N> {
    fn from(value: [T; N]) -> Self {
        Self { data: value }
    }
}

#[cfg(test)]
mod constructor_tests {
    use super::*;

    #[test]
    fn from_primitive() {
        let instance = Fixed::from([0, 1, 2, 3, 4]);
        assert_eq!(instance.data, [0, 1, 2, 3, 4]);
    }
}

impl<'a, T: 'a, const N: usize> Collection<'a> for Fixed<T, N> {
    type Element = T;

    fn count(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod collection_tests {
    use super::*;

    #[test]
    fn count() {
        let instance = Fixed::from([0, 1, 2, 3, 4]);
        assert_eq!(instance.count(), 5);
    }
}

impl<'a, T: 'a, const N: usize> Linear<'a> for Fixed<T, N> {
    fn iter(&self) -> impl std::iter::Iterator<Item = &'a Self::Element> {
        iter::Iter::new(self)
    }

    fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = &'a mut Self::Element> {
        iter::IterMut::new(self)
    }
}

#[cfg(test)]
mod iter_tests {
    use super::Fixed;

    #[test]
    fn immutable() {
        let array = Fixed::<usize, 4>::from([0, 1, 2, 3]);
        for (index, element) in array.iter().enumerate() {
            assert_eq!(index, *element);
        }
    }

    #[test]
    fn mutable() {
        let mut array: Fixed<usize, 16> = Default::default();
        for (index, element) in array.iter_mut().enumerate() {
            *element = index;
        }

        for (index, element) in array.iter().enumerate() {
            assert_eq!(index, *element);
        }
    }
}

impl<'a, T: 'a, const N: usize> Array<'a> for Fixed<T, N> {}

impl<T: Default, const N: usize> std::default::Default for Fixed<T, N> {
    fn default() -> Self {
        // SAFETY: the MaybeUninit it initalized even if the T isn't.
        let mut uninitalized: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for element in uninitalized.iter_mut() {
            element.write(Default::default());
        }

        // SAFETY:
        // * MaybeUninit<T> has same size as T => arrays have same size
        // * MaybeUninit<T> has same alignment as T => elements aligned
        let initalized = unsafe { uninitalized.as_mut_ptr().cast::<[T; N]>().read() };

        Self::from(initalized)
    }
}

#[cfg(test)]
mod default_tests {
    use super::*;

    #[test]
    fn zero_elements() {
        let array: Fixed<i32, 0> = Default::default();
        assert_eq!(array.count(), 0);
    }

    #[test]
    fn one_elements() {
        let array: Fixed<i32, 1> = Default::default();
        assert_eq!(array.count(), 1);
        assert_eq!(array[0], Default::default());
    }

    #[test]
    fn multiple_elements() {
        let array: Fixed<i32, 256> = Default::default();
        assert_eq!(array.count(), 256);
        for element in array {
            assert_eq!(element, Default::default());
        }
    }
}

impl<T: PartialEq, const N: usize> PartialEq for Fixed<T, N> {
    fn eq(&self, other: &Self) -> bool {
        if self.count() != other.count() {
            return false;
        }

        for (ours, theirs) in self.iter().zip(other.iter()) {
            if ours != theirs {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod partialeq_tests {
    use super::*;

    #[test]
    fn equality() {
        assert_eq!(Fixed::from([0, 1, 2]), Fixed::from([0, 1, 2]));
    }

    #[test]
    fn inequality() {
        assert_ne!(Fixed::from([0, 1, 3]), Fixed::from([0, 1, 2]));
    }
}

impl<T: Eq, const N: usize> std::cmp::Eq for Fixed<T, N> {}

impl<T: Clone, const N: usize> Clone for Fixed<T, N> {
    fn clone(&self) -> Self {
        // SAFETY: the MaybeUninit it initalized even if the T isn't.
        let mut uninitalized: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for (from, to) in self.iter().zip(uninitalized.iter_mut()) {
            to.write(from.clone());
        }

        // SAFETY:
        // * MaybeUninit<T> has same size as T => arrays have same size
        // * MaybeUninit<T> has same alignment as T => elements aligned
        let initalized = unsafe { uninitalized.as_mut_ptr().cast::<[T; N]>().read() };

        Self::from(initalized)
    }
}

#[cfg(test)]
mod clone_tests {
    use super::*;

    #[test]
    fn clone() {
        let old = Fixed::from([0,1,2,3]);
        let from = old.clone();
        assert_eq!(old, from);
    }
}

impl<T: Copy, const N: usize> Copy for Fixed<T, N> {}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for Fixed<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fixed").field("data", &self.data).finish()
    }
}

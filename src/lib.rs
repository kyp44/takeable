// Copyright 2017 Mathias Svensson. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT> or the Unlicense
// <LICENSE-UNLICENSE or https://unlicense.org/UNLICENSE>, at your option.
// All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Crate for temporarily moving out of a mutable pointer.
//!
//! This crate implementation a single wrapper-type `Takeable<T>`. The main purpose of this wrapper
//! is that it provides two convenient helper function `borrow` and `borrow_result` that allows for
//! temporarily moving out of the wrapper without violating safety.
//!
//! These work similarly to [`take`][take] from the [`take_mut`][take_mut] crate. The main
//! difference is that while the `take_mut` is implemented using careful handling of unwind safety,
//! this crate using an `Option<T>` inside to make unwinding work as expected.
//!
//! [take]: https://docs.rs/take_mut/0.1.3/take_mut/fn.take.html
//! [take_mut]: https://crates.io/crates/take_mut
#![no_std]

#![deny(missing_docs, unsafe_code,
        missing_debug_implementations, missing_copy_implementations,
        unstable_features, unused_import_braces, unused_qualifications)]

use core::ops::{Deref, DerefMut};

/// A wrapper-type that always holds a single `T` value.
///
/// This type is implemented using an `Option<T>`, however outside of the `borrow_result` function,
/// this `Option` will always contain a value.
///
/// # Panics
///
/// If the closure given to `borrow` or `borrow_result` panics, then the `Takeable` is left in an
/// unusable state without holding a `T`. Calling any method on the object besides `is_usable` when
/// in this state will result in a panic. This includes trying to dereference the object.
///
/// It is still safe to drop the value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Takeable<T> {
    // During normal usage, the invariant is that that this value should *always* be a Some(value),
    // unless we are inside the `borrow_result` function. However if the closure given the
    // `borrow_result` panics, then this will no longer be the case.
    value: Option<T>,
}

impl<T> Takeable<T> {
    /// Constructs a new `Takeable<T>` value.
    #[inline(always)]
    pub fn new(value: T) -> Takeable<T> {
        Takeable { value: Some(value) }
    }

    /// Gets a reference to the inner value.
    #[inline(always)]
    pub fn as_ref(&self) -> &T {
        self.value.as_ref().expect(
            "Takeable is not usable after a panic occurred in borrow or borrow_result",
        )
    }

    /// Gets a mutable reference to the inner value.
    #[inline(always)]
    pub fn as_mut(&mut self) -> &mut T {
        self.value.as_mut().expect(
            "Takeable is not usable after a panic occurred in borrow or borrow_result",
        )
    }

    /// Takes ownership of the inner value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.value.expect(
            "Takeable is not usable after a panic occurred in borrow or borrow_result",
        )
    }

    /// Updates the inner value using the provided closure.
    #[inline(always)]
    pub fn borrow<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        self.borrow_result(|v| (f(v), ()))
    }

    /// Updates the inner value using the provided closure while also returns a result.
    #[inline(always)]
    pub fn borrow_result<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(T) -> (T, R),
    {
        let old = self.value.take().expect(
            "Takeable is not usable after a panic occurred in borrow or borrow_result",
        );
        let (new, result) = f(old);
        self.value = Some(new);
        result
    }

    /// Check if the object is still usable.
    ///
    /// The object will always start out as usable, and can only enter an unusable state if the
    /// methods `borrow` or `borrow_result` are called with closures that panic.
    #[inline(always)]
    pub fn is_usable(&self) -> bool {
        self.value.is_some()
    }
}

impl<T> Deref for Takeable<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for Takeable<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::Takeable;
    #[test]
    fn test_takeable() {
        let mut takeable = Takeable::new(42u32);
        *takeable += 1;
        assert_eq!(*takeable, 43);
        *takeable.as_mut() += 1;
        assert_eq!(takeable.as_ref(), &44);
        takeable.borrow(|n: u32| n + 1);
        assert_eq!(*takeable, 45);
        let out = takeable.borrow_result(|n: u32| (n + 1, n));
        assert_eq!(out, 45);
        assert_eq!(takeable.into_inner(), 46);
    }

    #[test]
    #[should_panic]
    fn test_usable() {
        struct MyDrop {
            value: Takeable<()>,
            should_be_usable: bool,
        };
        impl Drop for MyDrop {
            fn drop(&mut self) {
                assert_eq!(self.value.is_usable(), self.should_be_usable);
            }
        }

        let _drop1 = MyDrop {
            value: Takeable::new(()),
            should_be_usable: true,
        };
        let mut drop2 = MyDrop {
            value: Takeable::new(()),
            should_be_usable: false,
        };
        drop2.value.borrow(|_| panic!());
    }
}

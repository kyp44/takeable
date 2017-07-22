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
//!
//! To do so effeciently, it uses the [`UncheckedOptionExt`][UncheckedOptionExt] trait from the
//! [`unreachable`][unreachable] crate. This behavior can be turned off by disabling the
//! `microoptimizations` feature.  `Option` at a lower performance cost.
//!
//! [UncheckedOptionExt]: https://docs.rs/unreachable/1.0.0/unreachable/trait.UncheckedOptionExt.html
//! [unreachable]: https://crates.io/crates/unreachable

#![no_std]

#![cfg_attr(not(feature = "microoptimization"), deny(unsafe_code))]
#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        unstable_features, unused_import_braces, unused_qualifications)]

#[cfg(feature = "microoptimization")]
extern crate unreachable;

use core::ops::{Deref, DerefMut};

#[cfg(feature = "microoptimization")]
#[path="unsafe_primitives.rs"]
mod primitives;

#[cfg(not(feature = "microoptimization"))]
#[path="safe_primitives.rs"]
mod primitives;

/// A wrapper-type that always hold a single `T` value.
///
/// This type is implemented using an `Option<T>`, however outside of the `borrow_result` function,
/// this Option will always contain a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Takeable<T> {
    // The invariant is that this value should *always* be a Some(value), unless we are inside the
    // `borrow_result` function.
    value: Option<T>,
}

impl<T> Takeable<T> {
    #[inline(always)]
    fn debug_assert_some(&self) {
        debug_assert!(self.value.is_some());
    }

    /// Constructs a new `Takeable<T>` value.
    #[inline(always)]
    pub fn new(value: T) -> Takeable<T> {
        Takeable { value: Some(value) }
    }

    /// Gets a reference to the inner value.
    #[inline(always)]
    pub fn as_ref(&self) -> &T {
        self.debug_assert_some();
        primitives::unwrap_some(self.value.as_ref())
    }

    /// Gets a mutable reference to the inner value.
    #[inline(always)]
    pub fn as_mut(&mut self) -> &mut T {
        self.debug_assert_some();
        primitives::unwrap_some(self.value.as_mut())
    }

    /// Takes ownership of the inner value.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.debug_assert_some();
        primitives::unwrap_some(self.value)
    }

    /// Updates the inner value using the provided closure.
    #[inline(always)]
    pub fn borrow<F>(&mut self, f: F)
        where F: FnOnce(T) -> T
    {
        self.debug_assert_some();
        self.borrow_result(|v| (f(v), ()))
    }

    /// Updates the inner value using the provided closure while also returns a result.
    #[inline(always)]
    pub fn borrow_result<F, R>(&mut self, f: F) -> R
        where F: FnOnce(T) -> (T, R)
    {
        self.debug_assert_some();
        let old = primitives::unwrap_some(self.value.take());
        let (new, result) = f(old);
        primitives::write_none(&mut self.value, new);
        result
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
}

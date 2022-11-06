// Copyright 2017 Mathias Svensson. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT> or the Unlicense
// <LICENSE-UNLICENSE or https://unlicense.org/UNLICENSE>, at your option.
// All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Crate for temporarily or permanently moving out of a mutable pointer.
//!
//! This crate implements a single wrapper-type [`Takeable<T>`]. The main purpose of this wrapper
//! is that it provides two convenient helper functions [`Takeable::borrow`] and [`Takeable::borrow_result`]
//! that allows for temporarily moving out of the wrapper without violating safety. The value can also be permanently
//! moved out, invalidating the  container and causing a panic on any future attempts to access
//! the value.
//!
//! The [`Takeable::borrow`] and [`Takeable::borrow_result`] methods work similarly to
//! [`take`] from the [`take_mut`] crate. The main difference is that, while the [`take_mut`]
//! is implemented using careful handling of unwind safety, this crate uses an [`Option<T>`] inside to make
//! unwinding work as expected.
//!
//! The [`Takeable::take`] method works similarly to [`Option::take`], but has the advantage that the object becomes
//! permanently invalidated. Additionally, using a [`Takeable<T>`] instead of an [`Option<T>`] makes
//! it clear in code that a value is always expected and avoids the need to handle possible
//! [`Option::None`] variants when accessing the `T`.
//!
//! [`take`]: https://docs.rs/take_mut/latest/take_mut/fn.take.html
//! [`take_mut`]: https://crates.io/crates/take_mut
#![no_std]
#![deny(
    missing_docs,
    unsafe_code,
    missing_debug_implementations,
    missing_copy_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use core::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// A wrapper-type that always holds a single `T` value.
///
/// This type is implemented using an [`Option<T>`], however, the wrapper requires the [`Option<T>`] to
/// always contain a value.
///
/// # Panics
///
/// If the closure given to [`borrow`][Self::borrow] or [`borrow_result`][Self::borrow_result] panics, then the `Takeable` is left in an
/// invalid state without holding a `T`. Calling any method on the object besides [`is_usable`][Self::is_usable] when
/// in this state will result in a panic. This includes trying to dereference the object. The object
/// will also be permanently invalidated if the value is moved out manually using [`take`][Self::take].
///
/// It is always safe to drop the `Takeable` even when invalidated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Takeable<T> {
    // During normal usage, the invariant is that that this value should *always* be a Some(value),
    // unless we are inside the `borrow_result` function. However if the closure given the
    // `borrow_result` panics, or the value is taken with `take`, then this will no longer be the
    // case.
    value: Option<T>,
}

/// Message to print when panicking because the `Takeable<T>` has been invalidated.
const PANIC_MESSAGE: &str = "the value has already been removed from the Takeable";

impl<T> Takeable<T> {
    /// Constructs a new [`Takeable<T>`] value.
    #[inline(always)]
    pub fn new(value: T) -> Takeable<T> {
        Takeable { value: Some(value) }
    }

    /// Takes ownership of the inner value.
    #[inline(always)]
    #[track_caller]
    pub fn into_inner(self) -> T {
        self.value.expect(PANIC_MESSAGE)
    }

    /// Updates the inner value using the provided closure.
    #[inline(always)]
    #[track_caller]
    pub fn borrow<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        self.borrow_result(|v| (f(v), ()))
    }

    /// Updates the inner value using the provided closure, which also returns a result.
    #[inline(always)]
    #[track_caller]
    pub fn borrow_result<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(T) -> (T, R),
    {
        let old = self.value.take().expect(PANIC_MESSAGE);
        let (new, result) = f(old);
        self.value = Some(new);
        result
    }

    /// Moves out the inner value and invalidates the object.
    ///
    /// Subsequent calls to any methods except [`is_usable`][Self::is_usable] will panic, including attempts to
    /// deference the object.
    #[inline(always)]
    #[track_caller]
    pub fn take(&mut self) -> T {
        self.value.take().expect(PANIC_MESSAGE)
    }

    /// Check if the object is still usable.
    ///
    /// The object will always start out as usable, and can only enter an unusable state if the
    /// methods [`borrow`][Self::borrow] or [`borrow_result`][Self::borrow_result] are called with closures that panic, or if the value
    /// is explicitly moved out permanently with [`take`][Self::take].
    #[inline(always)]
    pub fn is_usable(&self) -> bool {
        self.value.is_some()
    }
}

impl<T: Display> Display for Takeable<T> {
    #[inline(always)]
    #[track_caller]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<T> Deref for Takeable<T> {
    type Target = T;
    #[inline(always)]
    #[track_caller]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for Takeable<T> {
    #[inline(always)]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> From<T> for Takeable<T> {
    #[inline(always)]
    #[track_caller]
    /// Converts a `T` value into a [`Takeable<T>`].
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> AsRef<T> for Takeable<T> {
    #[inline(always)]
    #[track_caller]
    /// Gets a reference to the underlying value.
    fn as_ref(&self) -> &T {
        self.value.as_ref().expect(PANIC_MESSAGE)
    }
}

impl<T> AsMut<T> for Takeable<T> {
    #[inline(always)]
    #[track_caller]
    /// Gets a mutable reference to the underlying value.
    fn as_mut(&mut self) -> &mut T {
        self.value.as_mut().expect(PANIC_MESSAGE)
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
        let mut takeable = Takeable::new(34u32);
        assert_eq!(takeable.take(), 34);
    }

    #[test]
    #[should_panic]
    fn test_usable() {
        struct MyDrop {
            value: Takeable<()>,
            should_be_usable: bool,
        }
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
        let mut drop3 = MyDrop {
            value: Takeable::new(()),
            should_be_usable: false,
        };
        let _ = drop3.value.take();
        drop2.value.borrow(|_| panic!());
    }
}

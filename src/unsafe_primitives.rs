// Copyright 2017 Mathias Svensson. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT> or the Unlicense
// <LICENSE-UNLICENSE or https://unlicense.org/UNLICENSE>, at your option.
// All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use unreachable::UncheckedOptionExt;

#[inline(always)]
pub fn unwrap_some<T>(value: Option<T>) -> T {
    unsafe { value.unchecked_unwrap() }
}

#[inline(always)]
pub fn write_none<T>(loc: &mut Option<T>, value: T) {
    unsafe { loc.as_mut().unchecked_unwrap_none() };
    *loc = Some(value);
}

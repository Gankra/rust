// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic hashing support.
//!
//! This module provides a generic way to compute the hash of a value. The
//! simplest way to make a type hashable is to use `#[derive(Hash)]`:
//!
//! # Examples
//!
//! ```rust
//! use std::hash::{Hash, SipHasher, Hasher};
//!
//! #[derive(Hash)]
//! struct Person {
//!     id: u32,
//!     name: String,
//!     phone: u64,
//! }
//!
//! let person1 = Person { id: 5, name: "Janet".to_string(), phone: 555_666_7777 };
//! let person2 = Person { id: 5, name: "Bob".to_string(), phone: 555_666_7777 };
//!
//! assert!(hash(&person1) != hash(&person2));
//!
//! fn hash<T: Hash>(t: &T) -> u64 {
//!     let mut s = SipHasher::new();
//!     t.hash(&mut s);
//!     s.finish()
//! }
//! ```
//!
//! If you need more control over how a value is hashed, you need to implement
//! the trait `Hash`:
//!
//! ```rust
//! use std::hash::{Hash, Hasher, SipHasher};
//!
//! struct Person {
//!     id: u32,
//!     name: String,
//!     phone: u64,
//! }
//!
//! impl Hash for Person {
//!     fn hash<H: Hasher>(&self, state: &mut H) {
//!         self.id.hash(state);
//!         self.phone.hash(state);
//!     }
//! }
//!
//! let person1 = Person { id: 5, name: "Janet".to_string(), phone: 555_666_7777 };
//! let person2 = Person { id: 5, name: "Bob".to_string(), phone: 555_666_7777 };
//!
//! assert_eq!(hash(&person1), hash(&person2));
//!
//! fn hash<T: Hash>(t: &T) -> u64 {
//!     let mut s = SipHasher::new();
//!     t.hash(&mut s);
//!     s.finish()
//! }
//! ```

#![stable(feature = "rust1", since = "1.0.0")]

use prelude::v1::*;

use mem;

pub use self::sip::SipHasher;

mod sip;

/// A hashable type.
///
/// The `H` type parameter is an abstract hash state that is used by the `Hash`
/// to compute the hash.
///
/// If you are also implementing `Eq`, there is an additional property that
/// is important:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes should also be equal.
/// `HashMap` and `HashSet` both rely on this behavior.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Hash {
    /// Feeds this value into the state given, updating the hasher as necessary.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn hash<H: Hasher>(&self, state: &mut H);

    /// Feeds a slice of this type into the state provided.
    #[stable(feature = "hash_slice", since = "1.3.0")]
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where Self: Sized {
        for piece in data {
            piece.hash(state);
        }
    }

    /// Hashes only this value
    #[unstable(feature = "hash_one_shot", reason = "experimental", issue = "0")]
    fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
        self.hash(state);
        state.finish()
    }

    /// Hashes only this value
    #[unstable(feature = "hash_one_shot", reason = "experimental", issue = "0")]
    fn hash_slice_one_shot<H: Hasher>(data: &[Self], state: &mut H) -> u64
    where Self: Sized {
        Hash::hash_slice(data, state);
        state.finish()
    }
}

/// A trait which represents the ability to hash an arbitrary stream of bytes.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Hasher {
    /// Completes a round of hashing, producing the output hash generated.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn finish(&self) -> u64;

    /// Writes some data into this `Hasher`
    #[stable(feature = "rust1", since = "1.0.0")]
    fn write(&mut self, bytes: &[u8]);

    /// Writes only this data and then immediately finishes the hash.
    ///
    /// This enables hashers to more aggressively optimize
    #[unstable(feature = "hash_one_shot", reason = "experimental", issue = "0")]
    fn write_only(&mut self, bytes: &[u8]) -> u64 {
        self.write(bytes);
        self.finish()
    }

    /// Write a single `u8` into this hasher
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_u8(&mut self, i: u8) { self.write(&[i]) }
    /// Write a single `u16` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_u16(&mut self, i: u16) {
        self.write(&unsafe { mem::transmute::<_, [u8; 2]>(i) })
    }
    /// Write a single `u32` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_u32(&mut self, i: u32) {
        self.write(&unsafe { mem::transmute::<_, [u8; 4]>(i) })
    }
    /// Write a single `u64` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_u64(&mut self, i: u64) {
        self.write(&unsafe { mem::transmute::<_, [u8; 8]>(i) })
    }
    /// Write a single `usize` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_usize(&mut self, i: usize) {
        let bytes = unsafe {
            ::slice::from_raw_parts(&i as *const usize as *const u8,
                                    mem::size_of::<usize>())
        };
        self.write(bytes);
    }

    /// Write a single `i8` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_i8(&mut self, i: i8) { self.write_u8(i as u8) }
    /// Write a single `i16` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_i16(&mut self, i: i16) { self.write_u16(i as u16) }
    /// Write a single `i32` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_i32(&mut self, i: i32) { self.write_u32(i as u32) }
    /// Write a single `i64` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_i64(&mut self, i: i64) { self.write_u64(i as u64) }
    /// Write a single `isize` into this hasher.
    #[inline]
    #[stable(feature = "hasher_write", since = "1.3.0")]
    fn write_isize(&mut self, i: isize) { self.write_usize(i as usize) }
}

//////////////////////////////////////////////////////////////////////////////

mod impls {
    use prelude::v1::*;

    use slice;
    use super::*;

    macro_rules! impl_write {
        ($(($ty:ident, $meth:ident),)*) => {$(
            #[stable(feature = "rust1", since = "1.0.0")]
            impl Hash for $ty {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    state.$meth(*self)
                }

                fn hash_slice<H: Hasher>(data: &[$ty], state: &mut H) {
                    // FIXME(#23542) Replace with type ascription.
                    #![allow(trivial_casts)]
                    let newlen = data.len() * ::$ty::BYTES;
                    let ptr = data.as_ptr() as *const u8;
                    state.write(unsafe { slice::from_raw_parts(ptr, newlen) })
                }

                fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
                    Hash::hash_slice_one_shot(slice::ref_slice(self), state)
                }

                fn hash_slice_one_shot<H: Hasher>(data: &[$ty], state: &mut H) -> u64 {
                    // FIXME(#23542) Replace with type ascription.
                    #![allow(trivial_casts)]
                    let newlen = data.len() * ::$ty::BYTES;
                    let ptr = data.as_ptr() as *const u8;
                    state.write_only(unsafe { slice::from_raw_parts(ptr, newlen) })
                }
            }
        )*}
    }

    impl_write! {
        (u8, write_u8),
        (u16, write_u16),
        (u32, write_u32),
        (u64, write_u64),
        (usize, write_usize),
        (i8, write_i8),
        (i16, write_i16),
        (i32, write_i32),
        (i64, write_i64),
        (isize, write_isize),
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for bool {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u8(*self as u8)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (*self as u8).hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for char {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u32(*self as u32)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (*self as u32).hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Hash for str {
        fn hash<H: Hasher>(&self, state: &mut H) {
            // See `[T]` impl for why we write the u8
            state.write(self.as_bytes());
            state.write_u8(0xff)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            // See `[T] impl for why we *don't* write the u8
            self.as_bytes().hash_one_shot(state)
        }
    }

    macro_rules! impl_hash_tuple {
        () => (
            #[stable(feature = "rust1", since = "1.0.0")]
            impl Hash for () {
                fn hash<H: Hasher>(&self, _state: &mut H) {}
            }
        );

        ( $($name:ident)+) => (
            #[stable(feature = "rust1", since = "1.0.0")]
            impl<$($name: Hash),*> Hash for ($($name,)*) {
                #[allow(non_snake_case)]
                fn hash<S: Hasher>(&self, state: &mut S) {
                    let ($(ref $name,)*) = *self;
                    $($name.hash(state);)*
                }
            }
        );
    }

    impl_hash_tuple! {}
    // (A) is specialized below
    impl_hash_tuple! { A B }
    impl_hash_tuple! { A B C }
    impl_hash_tuple! { A B C D }
    impl_hash_tuple! { A B C D E }
    impl_hash_tuple! { A B C D E F }
    impl_hash_tuple! { A B C D E F G }
    impl_hash_tuple! { A B C D E F G H }
    impl_hash_tuple! { A B C D E F G H I }
    impl_hash_tuple! { A B C D E F G H I J }
    impl_hash_tuple! { A B C D E F G H I J K }
    impl_hash_tuple! { A B C D E F G H I J K L }

    impl<A: Hash> Hash for (A,) {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            self.0.hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T: Hash> Hash for [T] {
        fn hash<H: Hasher>(&self, state: &mut H) {
            // Hash in the `len` so ([a], [a, a]) and ([a, a], [a])
            // aren't hashed the same.
            self.len().hash(state);
            Hash::hash_slice(self, state)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            // Note: `len` not included here! No need to guard against
            // combinations; we're the only one being hashed!
            Hash::hash_slice_one_shot(self, state)
        }
    }


    #[stable(feature = "rust1", since = "1.0.0")]
    impl<'a, T: ?Sized + Hash> Hash for &'a T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            (**self).hash(state);
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (**self).hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<'a, T: ?Sized + Hash> Hash for &'a mut T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            (**self).hash(state);
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (**self).hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T> Hash for *const T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_usize(*self as usize)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (*self as usize).hash_one_shot(state)
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl<T> Hash for *mut T {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_usize(*self as usize)
        }

        fn hash_one_shot<H: Hasher>(&self, state: &mut H) -> u64 {
            (*self as usize).hash_one_shot(state)
        }
    }
}

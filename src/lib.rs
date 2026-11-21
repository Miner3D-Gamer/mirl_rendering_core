//! The core lib of `mirl_rendering`
#![allow(incomplete_features)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]
#![feature(const_clone)]
#![cfg_attr(feature = "font_support", feature(derive_const))]
#![cfg_attr(feature = "font_support", feature(const_cmp))]
// Additions
#![feature(specialization)]
#![feature(portable_simd)]

pub mod traits;

pub mod prelude;

pub mod cache;

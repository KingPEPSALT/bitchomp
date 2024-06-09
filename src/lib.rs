#![allow(incomplete_features)]
#![feature(
    generic_const_exprs,
    associated_type_defaults,
    negative_impls,
    trait_alias,
    debug_closure_helpers
)]

pub mod bytereader;
pub mod bytewriter;
pub mod transmutable;

#[cfg(test)]
mod test;
pub use bytereader::*;
pub use bytewriter::*;
pub use transmutable::*;

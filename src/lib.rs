#![allow(incomplete_features)]
#![feature(
    generic_const_exprs,
    associated_type_defaults,
    negative_impls,
    trait_alias,
    debug_closure_helpers
)]

mod bytereader;
mod bytewriter;
mod transmutable;
mod chomp;

#[cfg(test)]
mod test;

pub use bytereader::*;
pub use bytewriter::*;
pub use transmutable::*;
pub use chomp::*;
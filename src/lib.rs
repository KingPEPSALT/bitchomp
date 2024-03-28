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

#[cfg(test)]
mod test;

use bytereader::*;
use bytewriter::*;
use transmutable::*;

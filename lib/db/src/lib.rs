#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod tests;

pub mod methods;

pub mod wrapper;

use rusqlite::{Connection, Result};

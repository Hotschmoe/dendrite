//! Core library functions for testing.
const std = @import("std");

/// Adds two numbers together.
pub fn add(a: i32, b: i32) i32 {
    return a + b;
}

/// Maximum buffer size.
pub const MAX_SIZE: usize = 1024;

/// Multiplies two numbers.
pub fn multiply(a: i32, b: i32) i32 {
    return a * b;
}

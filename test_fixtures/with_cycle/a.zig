//! Module A - has cycle with B

const b = @import("b.zig");

pub fn funcA() void {
    b.funcB();
}

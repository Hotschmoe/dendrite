//! Module B - has cycle with A

const a = @import("a.zig");

pub fn funcB() void {
    a.funcA();
}

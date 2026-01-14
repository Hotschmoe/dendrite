//! Main entry point for the test application.
const std = @import("std");
const lib = @import("lib.zig");

pub fn main() !void {
    const result = lib.add(1, 2);
    std.debug.print("Result: {}\n", .{result});
}

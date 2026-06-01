// Fixture for testing generate if/else condition evaluation
module gen_if #(
    parameter FAST = 1,
    parameter WIDE = 0
) ();

    // Only the selected branch's instance should appear in the hierarchy.
    generate
        if (FAST) begin : gen_fast
            fast_core u_fast();
        end else begin : gen_slow
            slow_core u_slow();
        end
    endgenerate

    generate
        if (WIDE) begin : gen_wide
            wide_bus u_wide();
        end else begin : gen_narrow
            narrow_bus u_narrow();
        end
    endgenerate

endmodule

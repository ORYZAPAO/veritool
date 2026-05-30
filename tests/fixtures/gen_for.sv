// Fixture for testing generate for loop expansion
module gen_for #(
    parameter N = 4
) ();

    generate
        for (genvar i = 0; i < N; i++) begin : gen_blk
            unit_cell u_cell();
        end
    endgenerate

endmodule

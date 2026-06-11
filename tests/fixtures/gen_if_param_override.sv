// Fixture for testing generate-if branch selection reacting to instance
// parameter overrides (not just the module's own default parameters).
module wide_core (
    input  logic        clk,
    output logic [31:0] q
);
    always_ff @(posedge clk) begin
        q <= q + 1;
    end
endmodule

module narrow_core (
    input  logic       clk,
    output logic [7:0] q
);
    always_ff @(posedge clk) begin
        q <= q + 1;
    end
endmodule

module sel_core #(
    parameter WIDTH = 8
) (
    input  logic             clk,
    output logic [WIDTH-1:0] q
);
    generate
        if (WIDTH > 16) begin : gen_wide
            wide_core u_core (.clk(clk), .q(q[31:0]));
        end else begin : gen_narrow
            narrow_core u_core (.clk(clk), .q(q[7:0]));
        end
    endgenerate
endmodule

module gen_if_override_top ();
    // WIDTH=8 (<=16)  -> gen_narrow -> narrow_core (8 FFs)
    sel_core #(.WIDTH(8))  u_narrow ();
    // WIDTH=32 (>16)  -> gen_wide   -> wide_core (32 FFs)
    sel_core #(.WIDTH(32)) u_wide   ();
endmodule

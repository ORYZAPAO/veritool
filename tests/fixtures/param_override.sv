// Fixture for testing named parameter override extraction
module counter_w #(
    parameter WIDTH = 8
) (
    input  logic             clk,
    output logic [WIDTH-1:0] q
);
    always_ff @(posedge clk) begin
        q <= q + 1;
    end
endmodule

module param_top ();
    counter_w #(.WIDTH(16)) u_wide   ();
    counter_w #(.WIDTH(4))  u_narrow ();
endmodule

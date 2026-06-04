// Fixture for testing genvar value propagation in generate for loops.
// Each unit_w instance gets a different WIDTH based on the loop variable.
module unit_w #(parameter WIDTH = 1) ();
    logic [WIDTH-1:0] data;
    always_ff @(posedge clk) begin
        data <= data + 1;
    end
endmodule

// gen_for_param: N=3, generates unit_w with WIDTH = BASE + i
//   i=0: WIDTH=2 → 2 FFs
//   i=1: WIDTH=3 → 3 FFs
//   i=2: WIDTH=4 → 4 FFs
//   total own = 9
module gen_for_param #(
    parameter N    = 3,
    parameter BASE = 2
) ();
    genvar i;
    generate
        for (i = 0; i < N; i++) begin : gen_units
            unit_w #(.WIDTH(BASE + i)) u_unit();
        end
    endgenerate
endmodule

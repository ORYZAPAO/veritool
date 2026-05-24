// Simple 8-bit counter with enable
module counter #(
    parameter WIDTH = 8
) (
    input  logic             clk,
    input  logic             rst,
    input  logic             en,
    output logic [WIDTH-1:0] q
);

    always_ff @(posedge clk or posedge rst) begin
        if (rst)
            q <= '0;
        else if (en)
            q <= q + 1;
    end

endmodule

// Top module instantiating sub-modules

module alu (
    input  logic [7:0] a,
    input  logic [7:0] b,
    input  logic [1:0] op,
    output logic [7:0] result,
    output logic       zero
);
    logic [7:0] sum;
    logic [7:0] and_result;
    logic [7:0] or_result;

    assign sum        = a + b;
    assign and_result = a & b;
    assign or_result  = a | b;
    assign zero       = (result == 8'h00);

    always_ff @(posedge clk) begin
        case (op)
            2'b00: result <= sum;
            2'b01: result <= and_result;
            2'b10: result <= or_result;
            default: result <= a;
        endcase
    end
endmodule

module reg_file (
    input  logic        clk,
    input  logic        we,
    input  logic [2:0]  waddr,
    input  logic [7:0]  wdata,
    input  logic [2:0]  raddr,
    output logic [7:0]  rdata
);
    logic [7:0] regs [0:7];

    always_ff @(posedge clk) begin
        if (we)
            regs[waddr] <= wdata;
    end

    assign rdata = regs[raddr];
endmodule

module top1 (
    input  logic        clk,
    input  logic        rst,
    input  logic [7:0]  a,
    input  logic [7:0]  b,
    input  logic [1:0]  alu_op,
    output logic [7:0]  result
);
    logic [7:0] alu_out;
    logic [2:0] reg_waddr;
    logic       reg_we;

    alu u_alu (
        .a      (a),
        .b      (b),
        .op     (alu_op),
        .result (alu_out),
        .zero   ()
    );

    reg_file u_rf (
        .clk   (clk),
        .we    (reg_we),
        .waddr (reg_waddr),
        .wdata (alu_out),
        .raddr (reg_waddr),
        .rdata (result)
    );

    always_ff @(posedge clk or posedge rst) begin
        if (rst) begin
            reg_waddr <= '0;
            reg_we    <= 1'b0;
        end else begin
            reg_we    <= 1'b1;
            reg_waddr <= reg_waddr + 1;
        end
    end
endmodule

// Synchronous FIFO
module fifo_sync #(
    parameter WIDTH = 8,
    parameter DEPTH = 16
) (
    input  logic             clk,
    input  logic             rst,
    input  logic             wr_en,
    input  logic             rd_en,
    input  logic [WIDTH-1:0] wr_data,
    output logic [WIDTH-1:0] rd_data,
    output logic             full,
    output logic             empty
);

    localparam ADDR_W = $clog2(DEPTH);

    logic [WIDTH-1:0] mem [0:DEPTH-1];
    logic [ADDR_W:0]  wr_ptr;
    logic [ADDR_W:0]  rd_ptr;

    assign full  = (wr_ptr[ADDR_W] != rd_ptr[ADDR_W]) && (wr_ptr[ADDR_W-1:0] == rd_ptr[ADDR_W-1:0]);
    assign empty = (wr_ptr == rd_ptr);

    always_ff @(posedge clk or posedge rst) begin
        if (rst) begin
            wr_ptr <= '0;
            rd_ptr <= '0;
        end else begin
            if (wr_en && !full)
                wr_ptr <= wr_ptr + 1;
            if (rd_en && !empty)
                rd_ptr <= rd_ptr + 1;
        end
    end

    always_ff @(posedge clk) begin
        if (wr_en && !full)
            mem[wr_ptr[ADDR_W-1:0]] <= wr_data;
    end

    assign rd_data = mem[rd_ptr[ADDR_W-1:0]];

endmodule

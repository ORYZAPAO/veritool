// Fixture for testing generate case evaluation
module gen_case #(
    parameter MODE = 1
) ();

    generate
        case (MODE)
            0: begin : gen_small
                small_core u_small();
            end
            1: begin : gen_medium
                medium_core u_medium();
            end
            default: begin : gen_large
                large_core u_large();
            end
        endcase
    endgenerate

endmodule

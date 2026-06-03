// Fixture for testing generate casez/casex evaluation
// casez: ?, z, Z are wildcards
// casex: ?, z, Z, x, X are wildcards

// ── casez with binary wildcard ────────────────────────────────────────────────
// CTRL = 2 (2'b10) → matches 2'b1? (bit[0] is don't-care)
module gen_casez_bin #(
    parameter [1:0] CTRL = 2'b10
) ();
    generate
        casez (CTRL)
            2'b0?: begin : gen_slow  slow_core  u_slow();  end
            2'b1?: begin : gen_fast  fast_core  u_fast();  end
            default:       begin : gen_def   def_core   u_def();   end
        endcase
    endgenerate
endmodule

// ── casez with hex wildcard ───────────────────────────────────────────────────
// SEL = 8'hA0 (0xA0) → matches 8'hA? (lower nibble don't-care)
module gen_casez_hex #(
    parameter [7:0] SEL = 8'hA3
) ();
    generate
        casez (SEL)
            8'h1?: begin : gen_low    low_core   u_low();  end
            8'hA?: begin : gen_high   high_core  u_high(); end
            default:       begin : gen_other  other_core u_other(); end
        endcase
    endgenerate
endmodule

// ── casex with x wildcard ─────────────────────────────────────────────────────
// MODE = 4'b1010 → matches 4'bX0X0 (bits 3,1 are don't-care in casex)
module gen_casex_bin #(
    parameter [3:0] MODE = 4'b1010
) ();
    generate
        casex (MODE)
            4'bX1X1: begin : gen_odd   odd_core  u_odd();  end
            4'bX0X0: begin : gen_even  even_core u_even(); end
            default:         begin : gen_other other_core u_other(); end
        endcase
    endgenerate
endmodule

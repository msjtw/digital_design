module extend (
    input  logic [31:7] instr,
    input  logic [ 2:0] imm_src,
    output logic [31:0] imm_ext
);

  always_comb begin
    case (imm_src)
      // I-type
      'b000:   imm_ext = {{20{instr[31]}}, instr[31:20]};
      // S-type
      'b001:   imm_ext = {{20{instr[31]}}, instr[31:25], instr[11:7]};
      // B-type
      'b010:   imm_ext = {{20{instr[31]}}, instr[7], instr[30:25], instr[11:8], 1'b0};
      // U-type
      'b011:   imm_ext = {instr[31:12], 12'b0};
      // J-type
      'b011:   imm_ext = {{11{instr[31]}}, instr[31], instr[19:12], instr[20], instr[30:21], 1'b0};
      default: imm_ext = 'bx;
    endcase
  end

endmodule

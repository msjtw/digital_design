
module datapath (
    input logic clk,
    input logic reset,

    // Controller
    input logic PCWrite,
    AdrSrc,
    IRWrite,
    RegWrite,
    input logic [1:0] ResultSrc,
    ALUSrcA,
    ALUSrcB,
    ImmSrc,
    input logic [2:0] ALUControl,
    output logic zero,
    output logic [31:0] Instr,

    // Memory
    input  logic [31:0] ReadData,
    output logic [31:0] WriteData,
    Adress
);

  logic [31:0] Result;

  logic reg_wr_en;
  logic [31:0] reg_read_1, reg_read_2;
  reg_file rf (
      .clk(clk),
      .a1 (Instr[19:15]),
      .a2 (Instr[24:20]),
      .a3 (Instr[11:7]),
      .rd1(reg_read_1),
      .rd2(reg_read_2),
      .wd3(Result),
      .we3(reg_write_3)
  );

  logic [31:0] SrcA, SrcB, ALURes;
  logic [3:0] alu_flags;
  alu alu (
      .a(alu_a),
      .b(alu_b),
      .alu_control(ALUControl),
      .res(alu_res),
      .flags(alu_flags)
  );

  logic [31:0] PCNext, PC;
  floppr PC_reg (
      .clk(clk),
      .en (PCWrite),
      .d  (Result),
      .q  (PC)
  );

  mux4 adr_mux (
      .s ({0, AdrSrc}),
      .d0(PC),
      .d1(Result),
      .y (Adress)
  );


  logic [31:0] OldPC;
  floppr Instr_reg (
      .clk(clk),
      .en (IRWrite),
      .d  (ReadData),
      .q  (Instr)
  );

  floppr OldPC_reg (
      .clk(clk),
      .en (IRWrite),
      .d  (ReadData),
      .q  (OldPC)
  );

  floppr ReadData_reg (
      .clk(clk),
      .en (1),
      .d  (ReadData),
      .q  (Data)
  );

  logic [31:0] ImmExt;
  extend ext (
      .instr  (Instr[31:7]),
      .imm_src(ImmSrc),
      .imm_ext(ImmExt)
  );


  logic [31:0] A;
  floppr reg_floppr_1 (
      .clk(clk),
      .en (1),
      .d  (reg_read_1),
      .q  (A)
  );
  floppr reg_floppr_2 (
      .clk(clk),
      .en (1),
      .d  (reg_read_2),
      .q  (WriteData)
  );

  mux4 alu_mux_A (
      .d0(PC),
      .d1(OldPC),
      .d2(A),
      .s (ALUSrcA),
      .y (SrcA)
  );

  mux4 alu_mux_B (
      .d0(WriteData),
      .d1(ImmExt),
      .d2('d4),
      .s (ALUSrcB),
      .y (SrcB)
  );

  logic [31:0] ALUOut;
  floppr ALU_reg (
      .clk(clk),
      .en (1),
      .d  (ALURes),
      .q  (ALUOut)
  );

  mux4 res_mux (
      .d0(ALUOut),
      .d1(Data),
      .d2(ALURes),
      .s (ResultSrc),
      .y (Result)
  );

endmodule

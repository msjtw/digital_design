module riscvmulti (
    input logic clk,
    reset,
    input logic [31:0] ReadData,
    output logic write_enable,
    output logic [31:0] data_adr,
    WriteData
);
  logic zero;
  logic [31:0] Instr;
  logic PCWrite, AdrSrc, MemWrite, IRWrite, RegWrite;
  logic [1:0] ResultSrc, ALUSrcA, ALUSrcB, ImmSrc;
  logic [2:0] ALUControl;

  control_unit c (
      .clk(clk),
      .zero(zero),
      .Instr(Instr),
      .PCWrite(PCwrite),
      .AdrSrc(AdrSrc),
      .MemWrite(MemWrite),
      .IRWrite(IRWrite),
      .RegWrite(RegWrite),
      .ResultSrc(ResultSrc),
      .ALUSrcA(ALUSrcA),
      .ALUSrcB(ALUSrcA),
      .ImmSrc(ImmSrc),
      .ALUControl(ALUControl),
      .write_enable(write_enable)
  );

  datapath dp (
      .clk(clk),
      .reset(reset),
      .zero(zero),
      .Instr(Instr),
      .PCWrite(PCwrite),
      .AdrSrc(AdrSrc),
      .IRWrite(IRWrite),
      .RegWrite(RegWrite),
      .ResultSrc(ResultSrc),
      .ALUSrcA(ALUSrcA),
      .ALUSrcB(ALUSrcA),
      .ImmSrc(ImmSrc),
      .ALUControl(ALUControl),
      .ReadData(ReadData),
      .Adress(data_adr),
      .WriteData(WriteData)
  );

endmodule

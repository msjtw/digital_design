module datapath (
    input logic clk,
    input logic reset,

    // Controller
    input logic PCWrite,
    AdrSrc,
    MemWrite,
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

endmodule

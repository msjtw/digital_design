module control_unit (
    input logic clk,

    // Datapath
    input logic zero,
    input logic [31:0] Instr,
    output logic PCWrite,
    AdrSrc,
    MemWrite,
    IRWrite,
    RegWrite,
    output logic [1:0] ResultSrc,
    ALUSrcA,
    ALUSrcB,
    output logic [2:0] ALUControl,
    ImmSrc,

    // Memorty
    output logic write_enable
);

endmodule

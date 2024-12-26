module mem (
    input logic clk,
    input logic write_enable,
    input logic [31:0] WriteData,
    input logic [31:0] Adress,
    output logic [31:0] ReadData
);
  logic [31:0] RAM[64];

  initial $readmemh("riscv.txt", RAM);

  assign ReadData = RAM[Adress[31:2]];

  always_ff @(posedge clk) begin
    if (write_enable) RAM[Adress[31:2]] <= WriteData;
  end

endmodule

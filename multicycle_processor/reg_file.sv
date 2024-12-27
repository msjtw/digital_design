module reg_file (
    input logic clk,
    input logic we3,
    input logic [4:0] a1,
    a2,
    a3,
    input logic [31:0] wd3,
    output logic [31:0] rd1,
    rd2
);

  logic [31:0] regfile[32];

  always_ff @(posedge clk) begin
    if (we3) regfile[a3] <= wd3;
  end

  assign rd1 = (a1 == 0) ? 0 : regfile[a1];
  assign rd2 = (a2 == 0) ? 0 : regfile[a2];

endmodule


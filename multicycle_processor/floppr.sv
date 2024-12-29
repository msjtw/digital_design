module floppr #(
    parameter int WIDTH = 32
) (
    input logic clk,
    reset,
    en,
    input logic [WIDTH-1:0] d,
    output logic [WIDTH-1:0] q
);

  always_ff @(posedge clk, posedge reset) begin
    if (reset) q <= 0;
    else if (en) q <= d;
  end

endmodule

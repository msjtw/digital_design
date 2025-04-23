module shift_reg #(
    parameter N = 8
) (
    input logic clk,
    input logic load,
    input logic reset,
    input logic shift,
    input logic s_in,
    input logic [N-1:0] d,
    output logic s_out,
    output logic [N-1:0] q
);

  always_ff @(posedge clk, posedge reset)
    if (reset) q <= 0;
    else if (load) q <= d;
    else if (shift) q <= {q[N-2:0], s_in};

  assign s_out = q[N-1];

endmodule

module sr_tb ();
  logic clk, load, reset, shift, s_in, s_out;
  logic [7:0] d, q;

  shift_reg duv (
      .clk(clk),
      .load(load),
      .reset(reset),
      .shift(shift),
      .s_in(s_in),
      .d(d),
      .s_out(s_out),
      .q(q)
  );

  initial begin
    {reset, clk} = 0;
    d = 'b11111111;
    clk = 1;
    clk = 0;

  end

  always @(posedge clk) begin
    $display("%b", q);
  end

endmodule

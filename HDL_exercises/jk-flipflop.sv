module jk_ff (
    input  logic clk,
    input  logic j,
    k,
    output logic q
);

  always_ff @(posedge clk) begin
    if (j && k) q <= ~q;
    else if (j) q <= 1;
    else if (k) q <= 0;
  end

endmodule

module jk_tb ();

  logic j, k, clk, q;
  logic [2:0] dly;

  always begin
    clk = 1;
    #5;
    clk = 0;
    #5;
  end

  jk_ff duv (
      .clk(clk),
      .q  (q),
      .j  (j),
      .k  (k)
  );

  initial begin
    $dumpfile("vcd_dump/jk_ff.vcd");
    $dumpvars(0, jk_tb);
    $monitor("clk: %h, q: %h", clk, q);

    for (int i = 0; i < 10; i++) begin
      dly = $urandom;
      #(dly);
      j <= $urandom;
      dly = $urandom;
      #(dly);
      k <= $urandom;
    end

    #20 $finish(0);

  end

endmodule

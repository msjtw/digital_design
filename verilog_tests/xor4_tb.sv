/* verilator lint_off WIDTHTRUNC */

module xor4_tb ();

  logic a, b, c, d, y;

  xor4 dut (
      .a({a, b, c, d}),
      .y(y)
  );

  initial begin
    $monitor("%h%h%h%h %h", a, b, c, d, y);
    for (int i = 0; i < 16; i = i + 1) begin
      {a, b, c, d} = i;
      #1;
    end
  end
endmodule

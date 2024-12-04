module minority (
    input  logic a,
    b,
    c,
    output logic y
);

  assign y = ~a & ~b | ~a & ~c | ~b & ~c;

endmodule

module minority_tb ();
  logic a, b, c, y;
  minority dut (
      a,
      b,
      c,
      y
  );

  initial begin
    $monitor("%h%h%h %h", a, b, c, y);
    for (int i = 0; i < 8; i = i + 1) begin
      {a, b, c} = i;
      #1;
    end
  end
endmodule

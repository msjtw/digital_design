module func49_mux (
    input  logic a,
    b,
    c,
    output logic y
);

  mux8 m (
      .s({a, b, c}),
      .d(8'b00111001),
      .y(y)
  );
endmodule

module func (
    input  logic a,
    b,
    c,
    output logic y
);

  assign y = a & ~b | ~b & ~c | ~a & b & c;

endmodule

module func_tb ();
  logic [2:0] d;
  logic y1, y2;

  func49_mux f1_duv (
      .a(d[2]),
      .b(d[1]),
      .c(d[0]),
      .y(y1)
  );
  func49 f2_duv (
      .a(d[2]),
      .b(d[1]),
      .c(d[0]),
      .y(y2)
  );

  initial begin
    $monitor("%b -> %h : %h", d, y1, y2);
    for (int i = 0; i < 8; i++) begin
      d = i;
      #1;
    end
  end


endmodule

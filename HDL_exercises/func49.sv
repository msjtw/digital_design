module func49 (
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

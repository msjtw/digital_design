module mux8 (
    input logic [2:0] s,
    input logic [7:0] d,
    output logic y
);


  logic t0, t1;
  mux4 lower_mux (
      .s(s[1:0]),
      .d(d[3:0]),
      .y(t0)
  );
  mux4 upper_mux (
      .s(s[1:0]),
      .d(d[7:4]),
      .y(t1)
  );

  assign y = ~s[2] & t1 | s[2] & t0;

endmodule

module mux2 (
    input logic s,
    input logic [1:0] d,
    output logic y
);

  assign y = ~s & d[0] | s & d[1];

endmodule

module mux4 (
    input logic [1:0] s,
    input logic [3:0] d,
    output logic y
);

  assign y = s[1] ? (s[0] ? d[3] : d[2]) : (s[0] ? d[1] : d[0]);

endmodule

module mux8_tb ();
  logic [2:0] s;
  logic [7:0] d;
  logic y;
  mux8 duv (
      .s(s),
      .d(d),
      .y(y)
  );

  initial begin
    d = 8'b01010101;
    $monitor("%h %h %h", s, d, y);
    for (int i = 0; i < 8; i++) begin
      s = i;
      #1;
    end

  end

endmodule




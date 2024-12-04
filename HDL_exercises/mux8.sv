module mux2 (
    input logic s,
    d[2],
    output y
);

  assign y = s ? d[1] : d[0];

endmodule

module mux4 (
    input logic s[2],
    d[4],
    output y
);

  assign y = s[1] ? (s[0] ? d[3] : d[2]) : (s[0] ? d[1] : s[0]);

endmodule

module mux8 (
    input  logic s[3],
    d[8],
    output logic y
);
endmodule

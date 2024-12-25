module priority_circuit (
    input logic [7:0] d,
    output logic [7:0] y,
    output logic GS
);

  always_comb begin
    GS = |d;
    casez (d)
      'b1???????: y = 'b10000000;
      'b01??????: y = 'b01000000;
      'b001?????: y = 'b00100000;
      'b0001????: y = 'b00010000;
      'b00001???: y = 'b00001000;
      'b000001??: y = 'b00000100;
      'b0000001?: y = 'b00000010;
      'b00000001: y = 'b00000001;
      default: y = 0;
    endcase
  end

endmodule

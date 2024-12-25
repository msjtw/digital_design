module mul3 (
    input  logic clk,
    input  logic data,
    input  logic reset,
    output logic out
);

  //carry, out, last
  typedef enum logic [2:0] {
    S000,
    S001,
    S010,
    S011,
    S100,
    S101,
    S110,
    S111
  } statetype_e;

  statetype_e state, nextstate;

  always_ff @(posedge clk, negedge reset) begin
    if (~reset) state <= S000;
    else state <= nextstate;
  end

  //carry, out, last
  always_comb begin
    case (state)
      S000:
      if (data) nextstate = S011;
      else nextstate = S000;
      S010:
      if (data) nextstate = S011;
      else nextstate = S000;
      S011:
      if (data) nextstate = S101;
      else nextstate = S010;
      S100:
      if (data) nextstate = S101;
      else nextstate = S010;
      S101:
      if (data) nextstate = S111;
      else nextstate = S100;
      S111:
      if (data) nextstate = S111;
      else nextstate = S100;
      default: nextstate = S000;
    endcase
  end

  always_comb begin
    case (state)
      S000: out = 0;
      S100: out = 0;
      S101: out = 0;
      S010: out = 1;
      S011: out = 1;
      S111: out = 1;
      default: out = 0;
    endcase
  end

endmodule

module mul3_tb ();

  logic [7:0] data, out;
  logic d, o;
  logic clk, reset;

  always #10 clk = ~clk;

  mul3 duv (
      .clk  (clk),
      .reset(reset),
      .data (d),
      .out  (o)
  );

  initial begin
    for (int num = 0; num < 20; num++) begin
      {reset, clk} <= 0;
      data <= num;
      out <= 'b00000000;
      #5 reset <= 1;
      #3;
      for (int i = 0; i < 8; i++) begin
        d = data[i];
        #10;
        out[i] = o;
        #10;
      end

      #2;
      $display("%d * 3 = %d", data, out);
    end

    $finish(0);
  end


endmodule

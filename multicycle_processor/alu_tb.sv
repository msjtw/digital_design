
module alu_tb ();

  logic signed [31:0] a;
  logic signed [31:0] b;
  logic signed [31:0] res;
  logic [3:0] flags;
  logic [2:0] ctr;

  alu duv (
      .alu_control(ctr),
      .a(a),
      .b(b),
      .res(res),
      .flags(flags)
  );

  initial begin
    for (int i = -10; i < 10; i++) begin
      a = i;
      for (int k = -10; k < 10; k++) begin
        b   = k;
        ctr = 'b000;
        #10;
        $display("%d + %d = %d", a, b, res);
        ctr = 'b001;
        #10;
        $display("%d - %d = %d", a, b, res);
        ctr = 'b101;
        #10;
        $display("%d > %d = %d", a, b, res);
        ctr = 'b010;
        #10;
        $display("    %b\nand %b\n  = %b", a, b, res);
        ctr = 'b011;
        #10;
        $display("    %b\n or %b\n  = %b\n\n", a, b, res);


      end
    end
    $finish(0);
  end


endmodule

// alu control:
// 000  add
// 001  subtract
// 010  and
// 011  or
// 101  or


module alu (
    input logic [31:0] a,
    input logic [31:0] b,
    input logic [2:0] alu_control,
    output logic [31:0] res,
    output logic [3:0] flags,
    output logic overflow,
    carry,
    negative,
    zero
);

  logic [31:0] sum, cond_b;
  logic c_out;

  assign cond_b = alu_control[0] ? ~b : b;
  assign {c_out, sum} = a + cond_b + alu_control[0];
  assign carry = c_out & ~alu_control[1];
  assign overflow = ~(a[31] ^ b[31] ^ alu_control[0]) & (sum[31] & a[31]) & ~alu_control[1];
  assign zero = ~(&a);
  assign negative = res[31];
  assign flags = {overflow, carry, negative, zero};

  always_comb begin
    case (alu_control)
      'b000:   res = sum;
      'b001:   res = sum;
      'b010:   res = a & b;
      'b011:   res = a | b;
      'b101:   res = sum[31] ^ overflow;
      default: res = 32'bx;
    endcase
  end

endmodule



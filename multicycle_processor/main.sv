module main ();
  logic clk, reset;
  logic [31:0] write_data, read_data, data_adr;
  logic write_enable;

  riscvmulti rvmulti (
      .clk(clk),
      .reset(reset),
      .write_enable(write_enable),
      .data_adr(data_adr),
      .WriteData(write_data),
      .ReadData(read_data)
  );

  mem mem (
      .clk(clk),
      .Adress(data_adr),
      .write_enable(write_enable),
      .WriteData(write_data),
      .ReadData(read_data)
  );

endmodule

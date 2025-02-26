#!/bin/sh

iverilog -g2012 -o bin/rv multicycle_processor/alu.sv multicycle_processor/control_unit.sv multicycle_processor/datapath.sv multicycle_processor/extend.sv multicycle_processor/floppr.sv multicycle_processor/main.sv multicycle_processor/mem.sv multicycle_processor/mux4.sv multicycle_processor/reg_file.sv multicycle_processor/riscvmulti.sv

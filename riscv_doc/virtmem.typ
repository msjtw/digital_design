#import "@preview/cetz:0.4.0"
#import "@preview/cetz-plot:0.1.2"

#set page(
  numbering: "1"
)

#set heading(
  numbering: "1."
)

#pagebreak()
#align(center, text(17pt)[
  *RISC-V Virtual Memory*
])

= Introduction
Virtual to physical translation is made by hardware.
Each virtual address space has it's own page table.
Page table can be thought as a function:
$ f("vitual_address") arrow.r  "phisical_address, R/W/X/V/U" $
This function is used on all interactions with memory. 
To speed up the conversion a cache called *Translation Lookaside Buffer* can be used.

== Permissions:
- *X*: can fetch instruction from page?
- *V*: is valid?
- *U*: is user page? (instead of supervisor)

Users can only access user mode pages.
Kernel can only access supervisor mode pages.
Kernel can access user mode pages when bit *SUM* of *sstatus* is set. 
This is necessary to pass arguments and return values from system calls.


== Pages

Page size is 4kB ($2^12$ B), pages must align on page boundaries.
Page address:

#align(center,[
  #cetz.canvas({
    import cetz.draw: *
    rect((0, 0), (5, 0.5))
    rect((5, 0), (8, 0.5))
    content((0.25,0.8),"31")
    content((5-0.25,0.8),"12")
    content((5+0.25,0.8),"11")
    content((8-0.25,0.8),"0")
    content((2.5,0.25), "page num")
    content((6.5,0.25), "offset")
  })
])
Page table maps virtual page numbers to physical page numbers.

= Sv32
#align(center,[
  On RV32: 32-bit virtual address #sym.arrow.r 34-bit physical address
])

Each virtual address space can be 4GB, but the whole system can have 16GB of addressable memory.

To do the mapping radix trees are used (a tree in which each node has $n$ children, in Sv32 $n=1024$).

$ 1024 = 2^10", 2-level tree: " (2^10)^2 = 2^20 "leaves (number of virt pages)" $

#cetz.canvas({
  import cetz.draw: *
 
  let page_labels = ("0","1","2","3","⋯","1023")
  let page_table(xx, yy, n, lab) = {
    for i in range(0, 6) {
      let y = -i/3.4
      rect((xx+0, yy+y), (xx+1, yy+y+0.3), name: str(n)+"_"+str(i))
      if(lab){
        content((xx -0.1,yy+y+0.15), anchor: "east", page_labels.at(i))
      }
    }
  }

  page_table(0, 0, 0, false)
  page_table(0,-2.5, 1, false)
  page_table(-2.5,-2.5, 2, true)
  page_table(-5,-2.5, 3, true)

  line((2,0.3), "0_0.north-east" , mark: (end:">"), name: "satp")
  content("satp.start", anchor: "west", padding: .1, "satp points to the root of the tree")

  line(((2,-0.73)), "0_3.east" , mark: (end:">"), name: "root_pte")
  content("root_pte.start", anchor: "west", padding: .1, "internal PTE")

  line("0_0.center", "3_0.north", mark: (end:">"))
  line("0_1.center", "2_0.north", mark: (end:">"))
  line("0_2.center", "1_0.north", mark: (end:">"))

  rect((-1.25,-5), (rel: (0.5, 0.5)), name: "data1")
  rect((-0.25,-5), (rel: (0.5, 0.5)), name: "data2")
  rect((0.75,-5), (rel: (0.5, 0.5)), name: "data3")
  rect((1.75,-5), (rel: (0.5, 0.5)), name: "data4")

  line(("1_5.west",0.8,"1_5.east"), "data4.north", mark: (end:">"))
  line(("1_4.west",0.6,"1_4.east"), "data3.north", mark: (end:">"))
  line(("1_3.west",0.4,"1_3.east"), "data2.north", mark: (end:">"))
  line(("1_2.west",0.2,"1_2.east"), "data1.north", mark: (end:">"))

  line(((2,-3.23)), "1_3.east", mark: (end:">"), name: "pte")
  content("pte.start", anchor: "west", "leaf PTE, points to data pages")

  line(((3,-4.75)), "data4.east", mark: (end:">"), name: "data_p")
  content("data_p.start", anchor: "west", "data page")

})

Each node is an array of 1024 page table entries PTEs.
Each PTE points to a node at the next level.
Each entry is 4B(32 bit) so the whole node is 4kB, same as a page.

#pagebreak()

== Page table Entry PTE
Each node contains a PTE (page table entry).

#align(center,[
  #cetz.canvas({
    import cetz.draw: *
    import cetz-plot: *
    rect((0, 0), (5.5, 0.5), name: "ppn")
    content("ppn.center", "physical page number PPN")

    let labels = (
      "software", "dirty", "accessed", "global", "user page", "executable", "writable", "readable", "valid",
    )

    let bit_labels = "SSDAGUXWRV"
    for i in range(0, 10) {
      let x = 5 + i/2
      rect((x,0),(x+0.5, 0.5))
      content((x+0.25, 0.8), str(9-i))
      content((x+0.25, 0.25), bit_labels.at(i))
    }

    for i in range(1,10) {
      let pos = (i )/2
      content((3, -pos), labels.at(i -1), )
      line((4, -pos ), (5.25+i/2 ,-pos))
      line((5.25+i/2 ,-pos), (5.25+i/2, -.1), mark: (end:">"))
    }
    line((5.25, -0.5), (5.25, -0.1), mark: (end:">"))

  })
])

An interior page node entry (that doesn't point to actual data) has bits XWR set to 000.

== Address translation
- Virtual address:
#align(center,[
  #cetz.canvas({
    import cetz.draw: *
    rect((0, 0), (2.5, 0.5), name: "VPN1")
    rect((2.5, 0), (5, 0.5), name: "VPN0")
    rect((5, 0), (8, 0.5), name: "offset")
    content((2.5/2,0.8),"10 bits")
    content((1.5*2.5,0.8),"10 bits")
    content((5+(12/8),0.8),"12 bits")
    content((2.5/2,0.25),"VPN1")
    content((1.5*2.5,0.25),"VPN0")
    content((5+(12/8),0.25),"offset")

    line((0,-0.5), ((), "-|", "VPN1.south"), "VPN1.south", mark: (end:">"), name: "lvpn1")
    line((2,-1.2), ((), "-|", "VPN0.south"), "VPN0.south", mark: (end:">"), name: "lvpn0")

    content("lvpn1.start", anchor: "east", padding: 0.2, "index into\n root node")
    content("lvpn0.start", anchor: "east", padding: 0.2, "index into\n 2nd level")
  })
])
- satp register:
#align(center,[
  #cetz.canvas({
    import cetz.draw: *
    rect((0, 0), (0.5, 0.5), name: "mode")
    rect((0.5, 0), (10/4, 0.5), name: "asid")
    rect((10/4, 0), (8, 0.5), name: "ppn")
    content(((0.5/2),0.8),"1 bit")
    content((0.5+(10/4-0.5)/2,0.8),"9 bits")
    content((10/4+(8-10/4)/2,0.8),"22 bits")
    content("mode.center","M")
    content("asid.center","ASID")
    content("ppn.center","PPN")

    line((-0.5,-0.5), ((), "-|", "mode.south"), "mode.south", mark: (end:">"), name: "lmode")
    line((1,-1), ((), "-|", "asid.south"), "asid.south", mark: (end:">"), name: "lasid")
    line((2.5,-1.5), ((), "-|", "ppn.south"), "ppn.south", mark: (end:">"), name: "lppn")

    content("lmode.start", anchor: "east", padding: 0.2, "address translation scheme")
    content("lasid.start", anchor: "east", padding: 0.2, "address space id")
    content("lppn.start", anchor: "east", padding: 0.2, "physical page number")

  })
])

=== Algorithm

```
  address = satp[PPN] << 12 // address of the root node; address is 34 bit var
  // address of the root page from satp + offset from VPN1
  index = virtual_address[VPN1]·4
  pte = memory[address + index]

  address = pte[PPN]
  index = virtual_address[VPN0]·4
  pte = memory[address + index]

  // Atomically
  set pte[accessed]
  if operation is write:
    set pet[dirty]

  physical_address = pte[PPN] || virtual_memory[offset]
```

== Exceptions on page tree walk

/ Access Faults: problems accessing physical memory
/ Page Faults: problems accessing virtual memory

+ If the valid bit is not set  #sym.arrow page fault 
+ If W is set but R not #sym.arrow page fault
+ If proper bit for given instruction is not set #sym.arrow page fault

== Megapages

- If root PTE has bits R or X set it points to a megapage 
  - specification requires when W is set R is also set
  - it's sufficient to check bits R and X
- Megapage is 4MB in size, 1024 data pages combined.
- When translating a virtual address into mega page the 22 lsb  are used as offset.
- Megapages reduce the pressure on TLB
  - fewer entries are required to cover large amount of memory
  - Kernel is mapped into all address spaces, mapping it with megapages is more efficient

= Demand Paging


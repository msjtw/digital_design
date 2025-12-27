use super::virtio::*;


struct VirtioBlk{
    vblk: VirtioMmio<2>
}

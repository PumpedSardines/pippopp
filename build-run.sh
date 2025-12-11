./build.sh

# qemu-system-riscv32 -nographic -machine virt -m 128M -kernel ./build/main.bin
qemu-system-riscv32 -nographic -machine virt -m 512M \
  -smp 4 \
  -drive id=drive0,file=./data/drive0,format=raw,if=none,size=256M \
  -device virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0 \
  -monitor unix:qemu-monitor-socket,server,nowait \
  -kernel ./build/main.elf

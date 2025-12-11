RELEASE=false

if [ "$1" == "--release" ]; then
    RELEASE=true
fi

// Check release

if [ "$RELEASE" = true ]; then
  cargo build --release
else
  cargo build
fi

if [ $? -ne 0 ]; then
    exit 1
fi

if [ "$RELEASE" = true ]; then
  cp ./target/riscv32imac-unknown-none-elf/debug/osf ./main.elf
  objdump -D ./target/riscv32imac-unknown-none-elf/release/osf  > ./main.elf.txt
  objcopy -O binary ./target/riscv32imac-unknown-none-elf/release/osf ./main.bin
else
  cp ./target/riscv32imac-unknown-none-elf/debug/osf ./main.elf
  objdump -D ./target/riscv32imac-unknown-none-elf/debug/osf  > ./main.elf.txt
  objcopy -O binary ./target/riscv32imac-unknown-none-elf/debug/osf ./main.bin
fi

rm -rf build
mkdir -p build
mv main.bin build
mv main.elf.txt build
mv main.elf build

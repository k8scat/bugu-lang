test:
	cargo run -- example.bugu
	spike --isa=RV32G env/pk example

test2:
	cargo run -- example2.bugu
	spike --isa=RV32G env/pk example2
gcc:
	riscv-gcc/bin/riscv64-unknown-elf-gcc -march=rv32im -mabi=ilp32 example.S -o example_g
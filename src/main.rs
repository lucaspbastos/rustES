mod bus;
mod cpu;
mod ram;
fn main() {
    let mut cpu6502 = cpu::CPU::init();

    cpu6502.write_2_bytes_to_memory(0x8000, 0x0FA1);
    println!("{:?}", cpu6502.bus.memory);

    //cpu6502.interpret(vec![0xa9, 0x05, 0x00]);
}

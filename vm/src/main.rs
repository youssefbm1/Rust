fn main() -> Result<(), interpreter::Error> {
    let filename = std::env::args().nth(1).unwrap();
    let buffer = std::fs::read(filename).unwrap();
    let mut machine = interpreter::Machine::new(&buffer).unwrap();
    machine.run()
}

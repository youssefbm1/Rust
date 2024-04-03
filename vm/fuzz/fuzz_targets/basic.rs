#![no_main]
use interpreter::Machine;
use libfuzzer_sys::fuzz_target;

const MAX_STEPS: usize = 100_000;

fuzz_target!(|data: &[u8]| {
    let mut machine = Machine::new(data).unwrap();
    let mut o = Vec::new();
    for _ in 0..MAX_STEPS {
        match machine.step_on(&mut o) {
            Ok(true) | Err(_) => break,
            _ => (),
        }
    }
});

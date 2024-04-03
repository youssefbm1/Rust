#![no_main]
use interpreter::Machine;
use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

const MAX_STEPS: usize = 100_000;

const MEMORY_SIZE: usize = 4096;
const REGS: usize = 16;

struct MachineState {
    regs: [u32; REGS],
    small_rng_seed: u64,
    memory: Vec<u8>,
}

impl std::fmt::Debug for MachineState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = f.debug_struct("MachineState");
        s.field("regs", &self.regs)
            .field("small_rng_seed", &self.small_rng_seed)
            .finish()
    }
}

impl Arbitrary for MachineState {
    fn arbitrary(u: &mut Unstructured) -> arbitrary::Result<Self> {
        let regs = <[u32; REGS]>::arbitrary(u)?;
        let small_rng_seed = u64::arbitrary(u)?;
        let mut rng = SmallRng::seed_from_u64(small_rng_seed);

        let mut memory = Vec::with_capacity(MEMORY_SIZE);
        for _ in 0..MEMORY_SIZE / 8 {
            memory.extend(rng.next_u64().to_le_bytes());
        }
        Ok(Self {
            regs,
            small_rng_seed,
            memory,
        })
    }
}

fuzz_target!(|machine_state: MachineState| {
    let mut machine = Machine::new(&machine_state.memory).unwrap();
    for r in 0..REGS {
        machine.set_reg(r, machine_state.regs[r]).unwrap();
    }
    let mut o = Vec::new();
    for _ in 0..MAX_STEPS {
        match machine.step_on(&mut o) {
            Ok(true) | Err(_) => break,
            _ => (),
        }
    }
});

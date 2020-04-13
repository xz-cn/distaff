use std::time::{ Instant };
use crate::stark::{ TraceTable, prove };

pub mod opcodes;

const DEFAULT_EXTENSION_FACTOR: usize = 32;

pub fn execute(program: &[u64], inputs: &[u64], num_outputs: usize) -> ([u64; 4], Vec<u64>) {

    // pad the program to make sure the length is a power of two and the last operation is NOOP
    let mut program = program.to_vec();
    let trace_length = if program.len() == program.len().next_power_of_two() {
        program.len().next_power_of_two() * 2
    }
    else {
        program.len().next_power_of_two()
    };
    program.resize(trace_length, opcodes::NOOP);

    // execute the program to create an execution trace
    let now = Instant::now();
    let mut trace = TraceTable::new(&program, inputs, DEFAULT_EXTENSION_FACTOR);
    let t = now.elapsed().as_millis();
    println!("Generated execution trace of {} steps in {} ms", trace.len(), t);
    
    // copy the stack state the the last step to return as output
    let last_state = trace.get_state(trace.len() - 1);
    let outputs = last_state.get_stack()[0..num_outputs].to_vec();

    // copy the hash of the program
    let mut program_hash = [0u64; 4];
    program_hash.copy_from_slice(&last_state.get_op_acc()[0..4]);

    // generate STARK proof
    prove(&mut trace, inputs, &outputs);

    return (program_hash, outputs);
}
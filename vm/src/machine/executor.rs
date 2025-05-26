use super::architecture::VirtualMachineError;

pub trait Executor
{
    fn run(&mut self) -> Result<(), VirtualMachineError>;
    fn step(&mut self, cycles: usize) -> Result<(), VirtualMachineError>;
    fn stop(&mut self) -> Result<(), VirtualMachineError>;
    fn reset(&mut self) -> Result<(), VirtualMachineError>;
    fn delay(&mut self, millis: u64) -> Result<(), VirtualMachineError>;
}
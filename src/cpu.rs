use crate::instruction::{Instruction, InstructionData, instruction_set};
use crate::memory::MemoryChunk;

/// Gameboy clock state
#[derive(Default)]
pub struct Clock {
  m: u8,
  t: u8,
}

/// Represents a register pair that can be addressed either as two u8's or one u16
#[derive(Default)]
pub struct RegisterPair {
  l: u8,
  r: u8
}

impl RegisterPair {
  fn as_u16(&self) -> u16 {
    unimplemented!();
  }

  fn write_u16(&mut self, v: u16) {
    unimplemented!();
  }
}

/// Enum to address all the 8-bit registers
#[derive(Clone, Copy)]
pub enum SmallWidthRegister {
  B, C, A, F, D, E, H, L
}

/// Enum to address all the 16-bit wide registers
#[derive(Clone, Copy)]
pub enum WideRegister {
  PC, SP, BC, AF, DE, HL
}

/// CPU state and registers for a Z80 gameboy processor.
#[derive(Default)]
pub struct Registers {
  pc: u16,
  sp: u16,
  bc: RegisterPair,
  af: RegisterPair,
  de: RegisterPair,
  hl: RegisterPair,

  clock: Clock,
  interrupts_enabled: bool,
}

/// The position of the CARRY bit in the F (flags) register
const CARRY_BIT: u8 = 0x1;

impl Registers {

  pub fn pc(&self) -> u16 { self.read_r16(WideRegister::PC) } 
  pub fn sp(&self) -> u16 { self.read_r16(WideRegister::SP) }

  pub fn inc_pc(&mut self, by: u16) {
    self.write_r16(WideRegister::PC, self.read_r16(WideRegister::PC) + by);
  } 

  pub fn read_r8(&self, reg: SmallWidthRegister) -> u8 {
    match reg {
      B => self.bc.l,
      C => self.bc.r,
      A => self.af.l,
      F => self.af.r,
      D => self.de.l,
      E => self.de.r,
      H => self.hl.l,
      L => self.hl.r
    }
  }

  pub fn write_r8(&mut self, reg: SmallWidthRegister, val: u8) {
    match reg {
      B => self.bc.l = val,
      C => self.bc.r = val,
      A => self.af.l = val,
      F => self.af.r = val,
      D => self.de.l = val,
      E => self.de.r = val,
      H => self.hl.l = val,
      L => self.hl.r = val
    };
  }

  pub fn read_r16(&self, reg: WideRegister) -> u16 {
    match reg {
      PC => self.pc,
      SP => self.sp,
      BC => self.bc.as_u16(),
      AF => self.af.as_u16(),
      DE => self.de.as_u16(),
      HL => self.hl.as_u16()
    }
  }

  pub fn write_r16(&mut self, reg: WideRegister, val: u16) {
    match reg {
      PC => self.pc = val,
      SP => self.sp = val,
      BC => self.bc.write_u16(val),
      AF => self.af.write_u16(val),
      DE => self.de.write_u16(val),
      HL => self.hl.write_u16(val)
    };
  }

  pub fn carry(&self) -> bool {
    return self.read_r8(SmallWidthRegister::F) & CARRY_BIT != 0;
  }

  pub fn set_carry(&mut self, state: bool) {
    let mut current_flags = self.read_r8(SmallWidthRegister::F);
    if state {
      current_flags |= CARRY_BIT;
    } else {
      current_flags &= !CARRY_BIT;
    }
    self.write_r8(SmallWidthRegister::F, current_flags);
  }
}

pub struct CPU {
  registers: Registers,
  instructions: Vec<Instruction>
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      registers: Registers::default(),
      instructions: instruction_set(),
    }
  }
  fn instructions() -> Vec<Instruction> {
    instruction_set()
  } 
  pub fn step(&mut self, memory: &mut Box<dyn MemoryChunk>) {
    for i in 0..self.instructions.len() {
      println!("{}:{}", i, self.instructions[i].text);
    }
    let opcode = memory.read_u8(self.registers.pc());
    let inst = &CPU::instructions()[opcode as usize];
    (inst.execute)(&mut self.registers, memory, &inst.data);
  }
}
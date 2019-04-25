use std::env;
use std::fs;

fn main() {

    let filename = "astrododge.ch8";

    let contents = fs::read(filename)
        .expect("Something went wrong reading the file");

    let mut reg = RegisterBank::create();

    loop {
        let instruction = get_opcode(&contents, reg.pc as usize);
        reg.execute_instruction(instruction);

        //update screen

        //update timers
    }
}

fn get_opcode(memory: &Vec<u8>, index:usize) -> u16{
//    let mut opcode = [memory.get(index).unwrap(),memory.get(index+1).unwrap()];
    ((memory[index] as u16) << 8) | memory[index + 1] as u16
}

//Executes


#[derive(Debug)]
struct RegisterBank {
    reg: [u8; 16],
    reg_i: u16,
    pc: u16,
    delay: u8,
    sound: u8,
    stack: Vec<u16>,
}

impl RegisterBank {
    fn create() -> RegisterBank {
        RegisterBank {
            reg: [0; 16],
            reg_i: 0,
            pc: 0x200, //Thread for exec (Normally 60Hz could be faster)
            delay: 0,
            sound: 0, //Thread for sound (60 Hz, see spec)
            stack: Vec::new(),
            //Thread for visuals (same speed as exec thread)
        }
    }
    fn execute_instruction(&mut self, inst: u16){
        if inst  == 0x00E0 {
            println!("{:x?}, {:x?}", inst & 0x00E0, 0x00E0 );
            println!("Clear screen");
        } else if inst == 0x00EE {
            println!("{:x?}, {:x?}", inst & 0x00EE, 0x00EE );
            println!("Return");
        }else if inst & 0xF000 == 0x7000 {
            println!("ADD");
            let index = inst << 4 >> 12;
            let value = inst  << 8 >> 8;
            self.reg[index as usize] += value as u8;
            println!("instruction: {:x?}, RegisterBank: {:x?}, value: {:x?}", inst , index, value );
            println!("{:?}", self);
        }

        self.pc += 2; // increments program counter
        println!("Instruction: {:x?} next index:{:x?}", inst,self.pc);
    }
}

#[derive(Debug)]
enum Operation {
    /* x must be a nibble */
    ADD {x: u8, byte: u8},
    JMP {addr: u16},
}
impl Operation {
    fn parse(instruction:(u8,u8)) -> Operation {
        Operation::ADD{x: 1, byte: 0x07}
    }
}

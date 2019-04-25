use std::env;
use std::fs;

fn main() {
//    let mut reg: [u8; 16] = [0; 16];
//    let mut reg_i: u16 = 0;
//    let mut PC: u16 = 0;
//    let mut delay: u8;
//    let mut sound: u8;
//
    let mut stack: Vec<u16> = Vec::new();

    let args: Vec<String> = env::args().collect();

//    let query = &args[1];
    let filename = "astrododge.ch8";

    let contents = fs::read(filename)
        .expect("Something went wrong reading the file");

    let mut reg = Registry::create();

    loop {
        let instruction = get_opcode(&contents, reg.pc as usize);
        reg.execute_instruction(instruction);

        //update screen

        //update timers
    }


    stack.push(0xFF);
    stack.push(0x32);
    stack.push(0x1a);


    // reg[2] = 12;
    println!("{:?}", reg);
    println!("{}", stack.pop().unwrap());
    println!("{}", stack.pop().unwrap());
    println!("{}", stack.pop().unwrap());
}

fn get_opcode(memory: &Vec<u8>, index:usize) -> u16{
//    let mut opcode = [memory.get(index).unwrap(),memory.get(index+1).unwrap()];
    ((memory[index] as u16) << 8) | memory[index + 1] as u16
}

//Executes


#[derive(Debug)]
struct Registry {
    reg: [u8; 16],
    reg_i: u16,
    pc: u16,
    delay: u8,
    sound: u8,
    stack: Vec<u16>,
}

impl Registry {
    fn create() -> Registry {
        Registry {
            reg: [0; 16],
            reg_i: 0,
            pc: 0x200,
            delay: 0,
            sound: 0,
            stack: Vec::new(),
        }
    }
    fn execute_instruction(&mut self, inst: u16){
        if inst  == 0x00E0 {
            println!("{:x?}, {:x?}", inst & 0x00E0, 0x00E0 );
            println!("Clear screen");
        } else if inst == 0x00EE {
            println!("{:x?}, {:x?}", inst & 0x00EE, 0x00EE );
            println!("Return");
        }else if inst & 0x7000 == 0x7000 {
            println!("ADD");
            let index = inst << 4 >> 12;
            let value = inst  << 8 >> 8;
            self.reg[index as usize] += value as u8;
            println!("instruction: {:x?}, registry: {:x?}, value: {:x?}", inst , index, value );
            println!("{:?}", self);
        }

        self.pc += 2; // increments program counter
        println!("Instruction: {:x?} next index:{:x?}", inst,self.pc);
    }
}


//use std::env;
use std::fs;
use std::os::raw::c_void;

fn main() {
    let filename = "astrododge.ch8";

    let contents = fs::read(filename)
        .expect("Something went wrong reading the file");

    let mut reg = Resources::create();


    println!("{:?}", Operation::parse((&0x7a, &0x12)));

    loop {
        let instruction = get_opcode(&contents, reg.pc as usize);

        let op = Operation::parse((contents.get(reg.pc as usize).unwrap(), contents.get(reg.pc as usize).unwrap()));
        Operation::execute(&mut reg, op);
        reg.pc += 2;

        //reg.execute_instruction(instruction);

        //update screen

        //update timers
    }
}

fn get_opcode(memory: &Vec<u8>, index: usize) -> u16 {
//    let mut opcode = [memory.get(index).unwrap(),memory.get(index+1).unwrap()];
    ((memory[index] as u16) << 8) | memory[index + 1] as u16
}

//Executes


//#[derive(Debug)]
struct Resources {
    reg: [u8; 16],
    reg_i: u16,
    pc: u16,
    delay: u8,
    sound: u8,
    stack: Vec<u16>
}

impl Resources {
    fn create() -> Resources {
        Resources {
            reg: [0; 16],
            reg_i: 0,
            pc: 0x200, //Thread for exec (Normally 60Hz could be faster)
            delay: 0,
            sound: 0, //Thread for sound (60 Hz, see spec)
            stack: Vec::new()
            //Thread for visuals (same speed as exec thread)
        }
    }
    fn execute_instruction(&mut self, inst: u16) {
        if inst == 0x00E0 {
            println!("{:x?}, {:x?}", inst & 0x00E0, 0x00E0);
            println!("Clear screen");
        } else if inst == 0x00EE {
            println!("{:x?}, {:x?}", inst & 0x00EE, 0x00EE);
            println!("Return");
        } else if inst & 0xF000 == 0x7000 {
            println!("ADD");
            let index = inst << 4 >> 12;
            let value = inst << 8 >> 8;
            self.reg[index as usize] += value as u8;
            println!("instruction: {:x?}, RegisterBank: {:x?}, value: {:x?}", inst, index, value);
            println!("{:?}", self);
        }

        self.pc += 2; // increments program counter
        println!("Instruction: {:x?} next index:{:x?}", inst, self.pc);
    }
}

#[derive(Debug)]
enum Operation {
    /* x must be a nibble */
    SCDOWN { x: u8 },
    CLS,
    RTS,
    SCRIGHT,
    SCLEFT,
    LOW,
    HIGH,
    JMP { addr: u16 },
    JSR { addr: u16 },
    SKEQ_CONST { x: u8, byte: u8 },
    SKNE_CONST { x: u8, byte: u8 },
    SKEQ_REG { x: u8, y: u8 },
    MOV_CONST { x: u8, byte: u8 },
    ADD_CONST { x: u8, byte: u8 },
    MOV_REG { x: u8, y: u8 },
    OR { x: u8, y: u8 },
    AND_NO_CARRY { x: u8, y: u8 },
    XOR { x: u8, y: u8 },
    AND_CARRY { x: u8, y: u8 },
    SUB1_BORROW { x: u8, y: u8 },    // set vf to 1 if borrows Vx = Vx - Vy
    SHIFT_RIGHT { x: u8 },    //bit 0 -> Vf
    SUB2_BORROW { x: u8, y: u8 },    // set vf to 1 if borrows Vx = Vy - Vx
    SHIFT_LEFT { x: u8 },    // bit 7 -> Vf
    SKNE_REG { x: u8, y: u8 },
    MOV_I { value: u16 },
    JMP_I { addr: u16 },
    RAND { x: u8, max: u16 },
    SPRITE { x: u8, y: u8, s: u8 },
    XSPRITE { x: u8, y: u8 },
    SK_KEY_PRESS { key: u8 },
    SK_KEY_NOT_PRESS { key: u8 },
    GET_DELAY { x: u8 },
    KEY_WAIT { key: u8 },
    SET_DELAY { x: u8 },
    SET_SOUND { x: u8 },
    ADD_I { x: u8 },
    FONT { x: u8 },
    XFONT { x: u8 }, //Super only
    BCD { x: u8 },
    STORE_REG { x: u8 },
    LOAD_REG { x: u8 }
}

impl Operation {
    fn parse(instruction: (&u8, &u8)) -> Operation {
        match instruction {
            (0x00, 0xC0...0xCF) => Operation::SCDOWN { x: instruction.1 & 0x0F },
            (0x00, 0xE0) => Operation::CLS,
            (0x00, 0xEE) => Operation::RTS,
            (0x00, 0xFB) => Operation::SCRIGHT,
            (0x00, 0xFC) => Operation::SCLEFT,
            (0x00, 0xFE) => Operation::LOW,
            (0x00, 0xFF) => Operation::HIGH,
            (0x10...0x1F, _) => Operation::JMP { addr: ((instruction.0.clone() as u16) << 8) | instruction.1.clone() as u16 & 0x0FFF },
            (0x20...0x2F, _) => Operation::JSR { addr: ((instruction.0.clone() as u16) << 8) | instruction.1.clone() as u16 & 0x0FFF },
            (0x30...0x3F, _) => Operation::SKEQ_CONST { x: instruction.0 & 0x0F, byte: instruction.1.clone() },
            (0x40...0x4F, _) => Operation::SKNE_CONST { x: instruction.0 & 0x0F, byte: instruction.1.clone() },
            (0x50...0x5F, _) => Operation::SKEQ_REG { x: instruction.0 & 0x0F, y: instruction.1 >> 4 },
            (0x60...0x6F, _) => Operation::MOV_CONST { x: instruction.0 & 0x0F, byte: instruction.1.clone() },
            (0x70...0x7F, _) => Operation::ADD_CONST { x: instruction.0 & 0x0F, byte: instruction.1.clone() },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x00 => Operation::MOV_REG { x: instruction.0 & 0x0F, y: instruction.1 >> 4 },
            _ => Operation::JMP { addr: 0x200 }
        }
    }
    fn execute(resources: &mut Resources, opertation: Operation){
        match opertation {
            Operation::MOV_CONST { x:x, byte: b} => resources.reg[x as usize] = b,
            Operation::ADD_CONST { x:x, byte: b} => resources.reg[x as usize] += b,
            _ => ()
        }
    }
}

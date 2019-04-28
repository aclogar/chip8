use std::fs;

const MEMORY_SIZE: usize = 4096;
const STARTING_ADDR: u16 = 0x0200;

fn main() {
    let filename = "pong.ch8";

    let contents = fs::read(filename).expect("Something went wrong reading the file");

    let mut res = Resources::create();
    res.load_program(contents, STARTING_ADDR);
    println!("{:?}", Operation::parse(&(0x7a, 0x12)));

    loop {
        println!("Current Program Counter {:x?}", res.pc);
        //        let instruction = get_opcode(&contents, reg.pc as usize);
        //        println!("Current instruction {:x?}", instruction);
        let op = Operation::parse(&res.ram[res.pc as usize]);
        Operation::execute(&mut res, op);
        println!("RegisterBank: {:?}", res);
        res.pc += 1;

        //reg.execute_instruction(instruction);

        //update screen

        //update timers
    }
}

/* Resources available to the chip-8 emulator */
#[derive(Debug)]
struct Resources {
    reg: [u8; 16],
    reg_i: u16,
    pc: u16,
    delay: u8,
    sound: u8,
    stack: Vec<u16>,
    ram: Box<[(u8, u8)]>,
}

impl Resources {
    fn create() -> Resources {
        Resources {
            reg: [0; 16],
            reg_i: 0,
            pc: 0x200, //Thread for exec (Normally 60Hz could be faster)
            delay: 0,
            sound: 0, //Thread for sound (60 Hz, see spec)
            stack: Vec::new(),
            ram: Box::new([(0, 0); MEMORY_SIZE]),
            //Thread for visuals (same speed as exec thread)
        }
    }
    fn load_program(&mut self, mut program: Vec<u8>, start_index: u16) -> () {
        let mut cur_address: usize = start_index as usize;
        program.reverse();
        if program.len() % 2 != 0 {
            panic!("The program provided has an uneven number of bytes (ie missing a nibble)");
        }
        while let Some(byte1) = program.pop() {
            let byte2 = program.pop().expect("Failed to load second byte");
            self.ram[cur_address] = (byte1, byte2);
            cur_address += 1;
            if cur_address > std::u16::MAX as usize || cur_address > self.ram.len() {
                panic!("Program load address overflowed")
            } 
        }
    }
}

/* Operations allowed by the chip-8 emulator */
#[derive(Debug)]
enum Operation {
    /* x must be a nibble */
    Scdown { x: u8 },
    Cls,
    Rts,
    Scright,
    Scleft,
    Low,
    High,
    Jmp { addr: u16 },
    Jsr { addr: u16 },
    SkeqConst { x: u8, byte: u8 },
    SkneConst { x: u8, byte: u8 },
    SkeqReg { x: u8, y: u8 },
    MovConst { x: u8, byte: u8 },
    AddConst { x: u8, byte: u8 },
    MovReg { x: u8, y: u8 },
    _Or { x: u8, y: u8 },
    _And { x: u8, y: u8 },
    _Xor { x: u8, y: u8 },
    _AddReg { x: u8, y: u8 },
    _Sub1Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vx - Vy
    _ShiftRight { x: u8 },        //bit 0 -> Vf
    _Sub2Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vy - Vx
    _ShiftLeft { x: u8 },         // bit 7 -> Vf
    _SkneReg { x: u8, y: u8 },
    _MovI { value: u16 },
    _JmpI { addr: u16 },
    _Rand { x: u8, max: u8 },
    _Sprite { x: u8, y: u8, s: u8 },
    _XSprite { x: u8, y: u8 },
    _SkKeyPress { key: u8 },
    _SkKeyNotPress { key: u8 },
    _GetDelay { x: u8 },
    _KeyWait { key: u8 },
    _SetDelay { x: u8 },
    _SetSound { x: u8 },
    _AddI { x: u8 },
    _Font { x: u8 },
    _XFont { x: u8 }, //Super only
    _Bcd { x: u8 },
    _StoreReg { x: u8 },
    _LoadReg { x: u8 },
    NONE,
}

impl Operation {
    fn parse(instruction: &(u8, u8)) -> Operation {
        match instruction {
            (0x00, 0xC0...0xCF) => Operation::Scdown {
                x: instruction.1 & 0x0F,
            },
            (0x00, 0xE0) => Operation::Cls,
            (0x00, 0xEE) => Operation::Rts,
            (0x00, 0xFB) => Operation::Scright,
            (0x00, 0xFC) => Operation::Scleft,
            (0x00, 0xFE) => Operation::Low,
            (0x00, 0xFF) => Operation::High,
            (0x10...0x1F, _) => Operation::Jmp {
                addr: (((instruction.0.clone() as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0x20...0x2F, _) => Operation::Jsr {
                addr: (((instruction.0 as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0x30...0x3F, _) => Operation::SkeqConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1,
            },
            (0x40...0x4F, _) => Operation::SkneConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1,
            },
            (0x50...0x5F, _) => Operation::SkeqReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x60...0x6F, _) => Operation::MovConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1,
            },
            (0x70...0x7F, _) => Operation::AddConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x00 => Operation::MovReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x01 => Operation::_And {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x02 => Operation::_Or {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x03 => Operation::_Xor {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x04 => Operation::_AddReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x05 => Operation::_Sub1Borrow {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, 0x06) => Operation::_ShiftRight {
                x: instruction.0 & 0x0F,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x00 => Operation::_Sub2Borrow {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, 0x0E) => Operation::_ShiftLeft {
                x: instruction.0 & 0x0F,
            },
            (0x90...0x9F, _) if instruction.1 & 0x0F == 0x00 => Operation::_SkneReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0xA0...0xAF, _) => Operation::_MovI {
                value: (((instruction.0 as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0xB0...0xBF, _) => Operation::_JmpI {
                addr: (((instruction.0 as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0xC0...0xCF, _) => Operation::_Rand {
                x: instruction.0 & 0x0F,
                max:  instruction.1 ,
            },
            _ => Operation::NONE,
        }
    }
    fn execute(resources: &mut Resources, opertation: Operation) {
        match opertation {
            Operation::Rts => {
                println!("Returning");
                resources.pc = resources.stack.pop().unwrap()
            }
            Operation::Jsr { addr } => {
                resources.stack.push(resources.pc);
                resources.pc = addr ;
            }
            Operation::SkeqConst { x, byte } => {
                if resources.reg[x as usize] == byte {
                    resources.pc += 1;
                }
            }
            Operation::SkneConst { x, byte } => {
                if resources.reg[x as usize] != byte {
                    resources.pc += 1;
                }
            }
            Operation::SkeqReg { x, y } => {
                if resources.reg[x as usize] == resources.reg[y as usize] {
                    resources.pc += 1;
                }
            }
            Operation::MovConst { x, byte: b } => resources.reg[x as usize] = b,
            Operation::AddConst { x, byte: b } => resources.reg[x as usize] += b,
            Operation::MovReg {x, y}=> resources.reg[x as usize] = resources.reg[y as usize],
            Operation::_Or {x, y}=>  resources.reg[x as usize] |= resources.reg[y as usize],
            Operation::_And {x, y} =>  resources.reg[x as usize] &= resources.reg[y as usize],
            Operation::_Xor {x,y} =>  resources.reg[x as usize] ^= resources.reg[y as usize],
            Operation::_AddReg { x, y } => {
                resources.reg[x as usize] += resources.reg[y as usize];
                if resources.reg[x as usize] < x { resources.reg[0xf] = 1; }
            },
            Operation::_Sub1Borrow { x, y } => {
                if resources.reg[x as usize] > resources.reg[y as usize] { resources.reg[0xf] = 1; }
                resources.reg[x as usize] = resources.reg[x as usize] - resources.reg[y as usize];
            },
            Operation::_ShiftRight { x } => {
                resources.reg[0xf] = resources.reg[x as usize] & 0b0000_0001;
                resources.reg[x as usize] = resources.reg[x as usize] >> 1;
            },
            Operation::_Sub2Borrow { x, y } => {
                if resources.reg[y as usize] > resources.reg[x as usize] { resources.reg[0xf] = 1; }
                resources.reg[x as usize] = resources.reg[y as usize] - resources.reg[x as usize];
            },
            Operation::_ShiftLeft { x } => {
                resources.reg[0xf] = resources.reg[x as usize] & 0b1000_0000;
                resources.reg[x as usize] = resources.reg[x as usize] << 1;
            },
            Operation::_SkneReg { x, y } => {
                if resources.reg[x as usize] != resources.reg[y as usize] {
                    resources.pc += 1;
                }
            },
            Operation::_MovI { value } => resources.reg_i = value,
            _ => (),
        }
    }
}

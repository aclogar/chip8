use std::fs;

const MEMORY_SIZE: usize = 4096;
const STARTING_ADDR: u16 = 0x0200;

fn main() {
    let filename = "astrododge.ch8";

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
    _AndNoCarry { x: u8, y: u8 },
    _Xor { x: u8, y: u8 },
    _AndCarry { x: u8, y: u8 },
    _Sub1Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vx - Vy
    _ShiftRight { x: u8 },        //bit 0 -> Vf
    _Sub2Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vy - Vx
    _ShiftLeft { x: u8 },         // bit 7 -> Vf
    _SkneReg { x: u8, y: u8 },
    _MovI { value: u16 },
    _JmpI { addr: u16 },
    _Rand { x: u8, max: u16 },
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
                addr: (((instruction.0.clone() as u16) << 8) | instruction.1.clone() as u16)
                    & 0x0FFF,
            },
            (0x20...0x2F, _) => Operation::Jsr {
                addr: (((instruction.0.clone() as u16) << 8) | instruction.1.clone() as u16)
                    & 0x0FFF,
            },
            (0x30...0x3F, _) => Operation::SkeqConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1.clone(),
            },
            (0x40...0x4F, _) => Operation::SkneConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1.clone(),
            },
            (0x50...0x5F, _) => Operation::SkeqReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x60...0x6F, _) => Operation::MovConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1.clone(),
            },
            (0x70...0x7F, _) => Operation::AddConst {
                x: instruction.0 & 0x0F,
                byte: instruction.1.clone(),
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x00 => Operation::MovReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
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
                resources.pc = addr;
            }
            Operation::SkeqConst { x, byte } => {
                //                if
            }
            Operation::MovConst { x , byte: b } => resources.reg[x as usize] = b,
            Operation::AddConst { x, byte: b } => resources.reg[x as usize] += b,
            _ => (),
        }
    }
}

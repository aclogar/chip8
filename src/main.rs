extern crate rand;
extern crate sdl2;

use std::fs;
use sdl2::video::Window;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::pixels::Color;
use std::time::Duration;
use sdl2::keyboard::Keycode;
use std::process::exit;

const WINDOW_WIDTH : u32 = 640;
const WINDOW_HEIGHT : u32 = 320;

const MEMORY_SIZE: usize = 4096;
const STARTING_ADDR: u16 = 0x0200;

fn main() -> Result<(), String>{

    let filename = "pong.ch8";

    let contents = fs::read(filename).expect("Something went wrong reading the file");

    let mut res = Resources::create();
    res.load_program(contents, STARTING_ADDR);

    //Create display
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window = video_subsystem.window("Chip8 emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

//    res.ram[0xfFF] = 0x12;
//    res.ram[0xf12] = 0x12;

    'running: loop {

        let start_pc = res.pc;
        println!("Current Program Counter {:x?}", res.pc);
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { repeat: false, .. } => {
//                    keypress = true;
                },
                _ => {}
            }
        }

        //        let instruction = get_opcode(&contents, reg.pc as usize);
        //        println!("Current instruction {:x?}", instruction);
        let op = Operation::parse(&(res.ram[res.pc as usize], res.ram[(res.pc + 1) as usize]));
        Operation::execute(&mut res, op, );
        println!("RegisterBank: {:x?}", res);

        if start_pc == res.pc {
            res.pc += 2;
        }

        //reg.execute_instruction(instruction);

        //update screen
        update_screen(&mut window, &event_pump, &res);

        //update timers
//exit(0);
        if res.sound > 0 {res.sound -= 1;}
        if res.delay > 0 {res.delay -= 1;}
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}

fn update_screen(window: &mut Window, event_pump: &sdl2::EventPump, resources : &Resources) -> Result<(), String> {
    let mut surface = window.surface(event_pump)?;
    for i in 0xF00 ..= 0xFFF {
        let byte = resources.ram.get(i).unwrap();
        let index = i - 0xF00;
        for bit in 0..8 {
            let color = if (0x01 as u8) << bit & byte != 0 { Color::RGB(255, 0, 255) } else { Color::RGB(0, 0, 0) };

//            println!("I:{}, index:{}, X:{}, Y:{}", i, index, (((index % 8) * 8 + bit) * 10), ((index / 8) * 10));
            surface.fill_rect(Rect::new((((index % 8) * 8 + bit) * 10) as i32, ((index / 8) * 10) as i32, 10, 10), color)?;
        }
    }
    surface.finish()

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
    ram: Box<[u8]>,
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
            ram: Box::new([0; MEMORY_SIZE]),
            //Thread for visuals (same speed as exec thread)
        }
    }
    fn load_program(&mut self, mut program: Vec<u8>, start_index: u16) -> () {
        let mut cur_address: usize = start_index as usize;
        program.reverse();
        if program.len() % 2 != 0 {
            panic!("The program provided has an uneven number of bytes (ie missing a nibble)");
        }
        while let Some(byte) = program.pop() {
            self.ram[cur_address] = byte;
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
    Or { x: u8, y: u8 },
    And { x: u8, y: u8 },
    Xor { x: u8, y: u8 },
    AddReg { x: u8, y: u8 },
    Sub1Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vx - Vy
    ShiftRight { x: u8 },        //bit 0 -> Vf
    Sub2Borrow { x: u8, y: u8 }, // set vf to 1 if borrows Vx = Vy - Vx
    ShiftLeft { x: u8 },         // bit 7 -> Vf
    SkneReg { x: u8, y: u8 },
    MovI { value: u16 },
    JmpI { addr: u16 },
    Rand { x: u8, max: u8 },
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
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x01 => Operation::And {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x02 => Operation::Or {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x03 => Operation::Xor {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x04 => Operation::AddReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x05 => Operation::Sub1Borrow {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, 0x06) => Operation::ShiftRight {
                x: instruction.0 & 0x0F,
            },
            (0x80...0x8F, _) if instruction.1 & 0x0F == 0x00 => Operation::Sub2Borrow {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0x80...0x8F, 0x0E) => Operation::ShiftLeft {
                x: instruction.0 & 0x0F,
            },
            (0x90...0x9F, _) if instruction.1 & 0x0F == 0x00 => Operation::SkneReg {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
            },
            (0xA0...0xAF, _) => Operation::MovI {
                value: (((instruction.0 as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0xB0...0xBF, _) => Operation::JmpI {
                addr: (((instruction.0 as u16) << 8) | instruction.1 as u16)
                    & 0x0FFF,
            },
            (0xC0...0xCF, _) => Operation::Rand {
                x: instruction.0 & 0x0F,
                max: instruction.1,
            },
            (0xD0...0xDF, _) => Operation::_Sprite {
                x: instruction.0 & 0x0F,
                y: instruction.1 >> 4,
                s: instruction.1 & 0x0F,
            },
//            (0xE0...0xEF, 0x9E) => Operation::_SkKeyPress {
//                key: instruction.0 & 0x0F,
//            },
//            (0xE0...0xEF, 0xA1) => Operation::_SkKeyNotPress {
//                key: instruction.0 & 0x0F,
//            },
            (0xF0...0xFF, 0x07) => Operation::_GetDelay {
                x: instruction.0 & 0x0F,
            },
//            (0xE0...0xEF, 0xA1) => Operation::_KeyWait {
//                key: instruction.0 & 0x0F,
//            },
            (0xF0...0xFF, 0x15) => Operation::_SetDelay {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x18) => Operation::_SetSound {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x1E) => Operation::_AddI {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x29) => Operation::_Font {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x33) => Operation::_Bcd {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x55) => Operation::_StoreReg {
                x: instruction.0 & 0x0F,
            },
            (0xF0...0xFF, 0x65) => Operation::_LoadReg {
                x: instruction.0 & 0x0F,
            },
            _ => {
                println!("Passed instruction {:x?}", (((instruction.0 as u16) << 8) | instruction.1 as u16));
                Operation::NONE
            },
        }
    }
    fn execute(resources: &mut Resources, opertation: Operation) {
        match opertation {
            Operation::Rts => {
                println!("Returning");
                resources.pc = resources.stack.pop().unwrap()
            }

            Operation::Jmp { addr } => resources.pc = addr,
            Operation::Jsr { addr } => {
                resources.stack.push(resources.pc);
                resources.pc = addr ;
            }
            Operation::SkeqConst { x, byte } => {
                if resources.reg[x as usize] == byte {
                    resources.pc += 2;
                }
            }
            Operation::SkneConst { x, byte } => {
                if resources.reg[x as usize] != byte {
                    resources.pc += 2;
                }
            }
            Operation::SkeqReg { x, y } => {
                if resources.reg[x as usize] == resources.reg[y as usize] {
                    resources.pc += 2;
                }
            }
            Operation::MovConst { x, byte: b } => resources.reg[x as usize] = b,
            Operation::AddConst { x, byte: b } => resources.reg[x as usize] += b,
            Operation::MovReg {x, y}=> resources.reg[x as usize] = resources.reg[y as usize],
            Operation::Or {x, y}=>  resources.reg[x as usize] |= resources.reg[y as usize],
            Operation::And {x, y} =>  resources.reg[x as usize] &= resources.reg[y as usize],
            Operation::Xor {x,y} =>  resources.reg[x as usize] ^= resources.reg[y as usize],
            Operation::AddReg { x, y } => {
                resources.reg[x as usize] += resources.reg[y as usize];
                if resources.reg[x as usize] < x { resources.reg[0xf] = 1; }
            },
            Operation::Sub1Borrow { x, y } => {
                if resources.reg[x as usize] > resources.reg[y as usize] { resources.reg[0xf] = 1; }
                resources.reg[x as usize] = resources.reg[x as usize] - resources.reg[y as usize];
            },
            Operation::ShiftRight { x } => {
                resources.reg[0xf] = resources.reg[x as usize] & 0b0000_0001;
                resources.reg[x as usize] = resources.reg[x as usize] >> 1;
            },
            Operation::Sub2Borrow { x, y } => {
                if resources.reg[y as usize] > resources.reg[x as usize] { resources.reg[0xf] = 1; }
                resources.reg[x as usize] = resources.reg[y as usize] - resources.reg[x as usize];
            },
            Operation::ShiftLeft { x } => {
                resources.reg[0xf] = resources.reg[x as usize] & 0b1000_0000;
                resources.reg[x as usize] = resources.reg[x as usize] << 1;
            },
            Operation::SkneReg { x, y } => {
                if resources.reg[x as usize] != resources.reg[y as usize] {
                    resources.pc += 2;
                }
            },
            Operation::MovI { value } => resources.reg_i = value,
            Operation::JmpI { addr } => resources.pc = addr + resources.reg[0] as u16,
            Operation::Rand { x, max } => {
                resources.reg[x as usize] = rand::random::<u8>() & max;
            },
            Operation::_Sprite { x, y, s } => {
                let x = x as usize;
                let y = y as usize;
                let s = s as usize;

                for i in 0..= s {
                    println!("Draw at x:{}, y:{}", x, y+i);
                    resources.ram[(0xF00 + x + ((y+i)%32 * 4))] ^= resources.ram[resources.reg_i as usize + i];
                    //do bit flip detection
                }
            }
            Operation::_SetDelay { x } => resources.delay = resources.reg[x as usize],
            Operation::_GetDelay { x } => resources.reg[x as usize]= resources.delay,
            Operation::_SetSound {x} => resources.sound = resources.reg[x as usize],
            Operation::_AddI {x} => resources.reg_i += resources.reg[x as usize] as u16,
            Operation::_Font {x} => resources.reg_i = 0x0080 +  x as u16 * 5,
            Operation::_Bcd {x} => {
                println!("x.{}  :  {}, {}, {}", resources.reg[x as usize],resources.reg[x as usize] / 100, resources.reg[x as usize] / 10 % 10, resources.reg[x as usize] % 10);
                resources.ram[resources.reg_i as usize] = resources.reg[x as usize] / 100;
                resources.ram[resources.reg_i as usize + 1] = resources.reg[x as usize] % 100 / 10;
                resources.ram[resources.reg_i as usize + 2] = resources.reg[x as usize] % 10;
            },
            Operation::_StoreReg {x} => {
                for i  in 0..= x{
                    let i = i as usize;
                    let x = x as usize;
                    resources.ram[i + x] = resources.reg[x]
                }
            },
            Operation::_LoadReg {x} => {
                for i  in 0..= x{
                    let i = i as usize;
                    let x = x as usize;
                    resources.reg[x] = resources.ram[i + x];
                }
            },
            _ => {
                println!("Attempted to call an implemented Instruction {:?}", opertation);
            },
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn rand_test() {
        let op = Operation::parse(&(0xc1, 0x12));
        println!("{:?}", op);
        match op {
            Operation::Rand { x: 1, max: 0x12 } => (),
            _ => panic!("Invalid Operation selected")
        }
        let mut res = Resources::create();
        for _ in 0..50 {
            let op = Operation::parse(&(0xc1, 0x12));
            Operation::execute(&mut res, op);
            println!("{:x?}", res.reg[1]);
            assert!(res.reg[1] <= 0x12);
        }
    }
}

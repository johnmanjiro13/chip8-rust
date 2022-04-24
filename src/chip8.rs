use iced::time::every;
use iced::{executor, Application, Clipboard, Command, Element, Subscription};
use log::trace;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::display::Display;
use crate::memory::Memory;

struct Registers {
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
}

impl Registers {
    fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
        }
    }
}

#[derive(Default)]
struct Timers {
    dt: u8,
    st: u8,
}

pub struct Chip8 {
    registers: Registers,
    display: Display,
    memory: Memory,
    timers: Timers,
}

#[derive(Debug)]
pub enum Message {
    Display,
    Clock(Instant),
}

pub struct Flags {
    pub rom: Vec<u8>,
}

impl Application for Chip8 {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                registers: Registers::new(),
                display: Display::new(),
                memory: Memory::with_rom(flags.rom),
                timers: Timers::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Chip8")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Clock(_instance) => {
                let b1 = self.memory.load(self.registers.pc);
                let b2 = self.memory.load(self.registers.pc + 1);
                self.execute(b1 >> 4, b1 & 0x0F, b2 >> 4, b2 & 0x0F);
            }
            Message::Display => {}
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.display.view().map(|_| Message::Display)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = every(Duration::from_millis(1000 / 60)).map(Message::Clock);
        Subscription::batch([clock])
    }
}

impl Chip8 {
    fn execute(&mut self, h1: u8, h2: u8, h3: u8, h4: u8) {
        trace!(
            "PC={:04X}, I={:04X}, v={:?}",
            self.registers.pc,
            self.registers.i,
            self.registers.v
        );
        match (h1, h2, h3, h4) {
            (0x0, 0x0, 0xE, 0x0) => {
                trace!("{:04X} CLS", self.registers.pc);
                self.display.clear();
                self.registers.pc += 2;
            }
            (0x0, 0x0, 0xE, 0xE) => {
                trace!("{:04X} RET", self.registers.pc);
                self.registers.pc = self.registers.stack[self.registers.sp as usize];
                self.registers.sp -= 1;
                self.registers.pc += 2;
            }
            (0x1, n1, n2, n3) => {
                let addr = addr_of(n1, n2, n3);
                trace!("{:04X} JP {:04X}", self.registers.pc, addr);
                self.registers.pc = addr;
            }
            (0x2, n1, n2, n3) => {
                let addr = addr_of(n1, n2, n3);
                trace!("{:04X} CALL {:04X}", self.registers.pc, addr);
                self.registers.sp += 1;
                self.registers.stack[self.registers.sp as usize] = self.registers.pc;
                self.registers.pc = addr;
            }
            (0x3, x, k1, k2) => {
                let value = value_of(k1, k2);
                trace!("{:04X} SE V{:X}, {:02X}", self.registers.pc, x, value);
                if self.registers.v[x as usize] == value {
                    self.registers.pc += 4;
                } else {
                    self.registers.pc += 2;
                }
            }
            (0x4, x, k1, k2) => {
                let value = value_of(k1, k2);
                trace!("{:04X} SNE V{:X}, {:02X}", self.registers.pc, x, value);
                if self.registers.v[x as usize] != value {
                    self.registers.pc += 4;
                } else {
                    self.registers.pc += 2;
                }
            }
            (0x5, x, y, 0x0) => {
                trace!("{:04X} SE V{:X}, V{:X}", self.registers.pc, x, y);
                if self.registers.v[x as usize] == self.registers.v[y as usize] {
                    self.registers.pc += 4;
                } else {
                    self.registers.pc += 2;
                }
            }
            (0x6, x, k1, k2) => {
                let value = value_of(k1, k2);
                trace!("{:04X} LD V{:X}, {:02X}", self.registers.pc, x, value);
                self.registers.v[x as usize] = value;
                self.registers.pc += 2;
            }
            (0x7, x, k1, k2) => {
                let value = value_of(k1, k2);
                trace!("{:04X} ADD V{:X}, {:02X}", self.registers.pc, x, value);
                self.registers.v[x as usize] += value;
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x0) => {
                trace!("{:04X} LD V{:X}, V{:X}", self.registers.pc, x, y);
                self.registers.v[x as usize] = self.registers.v[y as usize];
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x1) => {
                trace!("{:04X} OR V{:X}, V{:X}", self.registers.pc, x, y);
                self.registers.v[x as usize] |= self.registers.v[y as usize];
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x2) => {
                trace!("{:04X} AND V{:X}, V{:X}", self.registers.pc, x, y);
                self.registers.v[x as usize] &= self.registers.v[y as usize];
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x3) => {
                trace!("{:04X} XOR V{:X}, V{:X}", self.registers.pc, x, y);
                self.registers.v[x as usize] ^= self.registers.v[y as usize];
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x4) => {
                trace!("{:04X} ADD V{:X}, V{:X}", self.registers.pc, x, y);
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (result, carry) = vx.overflowing_add(vy);
                self.registers.v[x as usize] = result;
                self.registers.v[0xF] = if carry { 0x01 } else { 0x00 };
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x5) => {
                trace!("{:04X} SUB V{:X}, V{:X}", self.registers.pc, x, y);
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (result, borrow) = vx.overflowing_sub(vy);
                self.registers.v[x as usize] = result;
                self.registers.v[0xF] = if !borrow { 0x01 } else { 0x00 };
                self.registers.pc += 2;
            }
            (0x8, x, _y, 0x6) => {
                trace!("{:04X} SHR V{:X}, {{V{:X}}}", self.registers.pc, x, _y);
                let vx = self.registers.v[x as usize];
                self.registers.v[0xF] = if vx % 2 == 1 { 0x01 } else { 0x00 };
                self.registers.v[x as usize] = vx >> 1;
                self.registers.pc += 2;
            }
            (0x8, x, y, 0x7) => {
                trace!("{:04X} SUBN V{:X}, V{:X}", self.registers.pc, x, y);
                let vx = self.registers.v[x as usize];
                let vy = self.registers.v[y as usize];
                let (result, borrow) = vy.overflowing_sub(vx);
                self.registers.v[x as usize] = result;
                self.registers.v[0xF] = if !borrow { 0x01 } else { 0x00 };
                self.registers.pc += 2;
            }
            (0x8, x, _y, 0xE) => {
                trace!("{:04X} SHL V{:X}, {{V{:X}}}", self.registers.pc, x, _y);
                let vx = self.registers.v[x as usize];
                self.registers.v[0xF] = if (vx >> 7) % 2 == 1 { 0x01 } else { 0x00 };
                self.registers.v[x as usize] = vx << 1;
                self.registers.pc += 2;
            }
            (0x9, x, y, 0x0) => {
                trace!("{:04X} SNE V{:X}, V{:X}", self.registers.pc, x, y);
                if self.registers.v[x as usize] != self.registers.v[y as usize] {
                    self.registers.pc += 4;
                } else {
                    self.registers.pc += 2;
                }
            }
            (0xA, n1, n2, n3) => {
                let addr = addr_of(n1, n2, n3);
                trace!("{:04X} LD, I, {:04X}", self.registers.pc, addr);
                self.registers.i = addr;
                self.registers.pc += 2;
            }
            (0xB, n1, n2, n3) => {
                let addr = addr_of(n1, n2, n3);
                trace!("{:04X} JP V0, {:04X}", self.registers.pc, addr);
                let v0 = self.registers.v[0x0];
                self.registers.pc = addr + v0 as u16;
            }
            (0xC, x, k1, k2) => {
                let value = value_of(k1, k2);
                trace!("{:04X} RND V{:X}, {:02X}", self.registers.pc, x, value);
                let mut rng = rand::thread_rng();
                let random = rng.gen_range(0..=0xFF);
                self.registers.v[x as usize] = random & value;
                self.registers.pc += 2;
            }
            (0xD, x, y, n) => {
                let from = self.registers.i;
                let sprite = self.memory.load_sprite(from, n);

                trace!(
                    "{:04X} DRW V{:X}, V{:X} {:X} (sprite: {:?})",
                    self.registers.pc,
                    x,
                    y,
                    n,
                    sprite
                );

                let corner_x = self.registers.v[x as usize];
                let corner_y = self.registers.v[y as usize];
                let collision = self.display.draw_sprite(corner_x, corner_y, sprite);
                self.registers.v[0xF] = if collision { 0x01 } else { 0x00 };
                self.registers.pc += 2;
            }
            (0xE, x, 0x9, 0xE) => {
                trace!("{:04X} SKP V{:X}", self.registers.pc, x);
                // TODO: keyboard
                self.registers.pc += 2;
            }
            (0xE, x, 0xA, 0x1) => {
                trace!("{:04X} SKNP V{:X}", self.registers.pc, x);
                // TODO: keyboard
                self.registers.pc += 2;
            }
            (0xF, x, 0x0, 0x7) => {
                trace!("{:04X} LD V{:X}, DT", self.registers.pc, x);
                self.registers.v[x as usize] = self.timers.dt;
                self.registers.pc += 2;
            }
            (0xF, x, 0x0, 0xA) => {
                trace!("{:04X} LD V{:X}, K", self.registers.pc, x);
                // TODO: keyboard
                self.registers.pc += 2;
            }
            (0xF, x, 0x1, 0x5) => {
                trace!("{:04X} LD DT, V{:X}", self.registers.pc, x);
                self.timers.dt = self.registers.v[x as usize];
                self.registers.pc += 2;
            }
            (0xF, x, 0x1, 0x8) => {
                trace!("{:04X} LD ST, V{:X}", self.registers.pc, x);
                self.timers.st = self.registers.v[x as usize];
                self.registers.pc += 2;
            }
            (0xF, x, 0x1, 0xE) => {
                trace!("{:04X} ADD I, V{:X}", self.registers.pc, x);
                self.registers.i += self.registers.v[x as usize] as u16;
                self.registers.pc += 2;
            }
            (0xF, x, 0x2, 0x9) => {
                trace!("{:04X} LD F, V{:X}", self.registers.pc, x);
                let font = self.registers.v[x as usize];
                self.registers.i = Memory::font_addr(font);
                self.registers.pc += 2;
            }
            (0xF, x, 0x3, 0x3) => {
                trace!("{:04X} LD B, V{:X}", self.registers.pc, x);
                let from = self.registers.i;
                let value = self.registers.v[x as usize];
                self.memory.store(from, value / 100);
                self.memory.store(from + 1, (value / 10) % 10);
                self.memory.store(from + 2, value % 10);
                self.registers.pc += 2;
            }
            (0xF, x, 0x5, 0x5) => {
                trace!("{:04X} LD [I], V{:X}", self.registers.pc, x);
                let from = self.registers.i;
                for offset in 0..=x {
                    let value = self.registers.v[offset as usize];
                    self.memory.store(from + offset as u16, value);
                }
                self.registers.pc += 2;
            }
            (0xF, x, 0x6, 0x5) => {
                trace!("{:04X} LD V{:X}, [I]", self.registers.pc, x);
                let from = self.registers.i;
                for offset in 0..=x {
                    let value = self.memory.load(from + offset as u16);
                    self.registers.v[offset as usize] = value;
                }
                self.registers.pc += 2;
            }
            _ => {
                panic!("unsupported inst: {:X}{:X}{:X}{:X}", h1, h2, h3, h4);
            }
        }
    }
}

fn value_of(n1: u8, n2: u8) -> u8 {
    (n1 << 4) | n2
}

fn addr_of(n1: u8, n2: u8, n3: u8) -> u16 {
    (n1 as u16) << 8 | (n2 as u16) << 4 | (n3 as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addr_of() {
        assert_eq!(addr_of(0x1, 0x2, 0x3), 0x123);
    }

    #[test]
    fn test_value_of() {
        assert_eq!(value_of(0x1, 0x2), 0x12);
    }
}

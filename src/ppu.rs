use crate::mmu::MMU;
use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
    pub mmu: Rc<RefCell<MMU>>,
    pub framebuffer: [u8; 160 * 144]
}

impl PPU {
    pub fn new(mmu: Rc<RefCell<MMU>>) -> PPU {
        let framebuffer = [0; 160 * 144];

        PPU {
            mmu: mmu,
            framebuffer: framebuffer
        }
    }

    pub fn update(&mut self, cycles: u32) {
        
    }
}
use core::cell::RefCell;
use core::ops::DerefMut;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

pub struct SharedDelay<D>
where
    D: DelayMs<u8> + DelayUs<u16>,
{
    delay: RefCell<D>,
}

pub struct SharedDelayHandler<'a, D>
where
    D: DelayMs<u8> + DelayUs<u16>,
{
    delay: &'a RefCell<D>,
}

impl<D> SharedDelay<D>
where
    D: DelayMs<u8> + DelayUs<u16>,
{
    pub fn new(delay: D) -> Self {
        SharedDelay {
            delay: RefCell::new(delay),
        }
    }

    pub fn share(&self) -> SharedDelayHandler<D> {
        SharedDelayHandler { delay: &self.delay }
    }
}

impl<D> DelayUs<u16> for SharedDelayHandler<'_, D>
where
    D: DelayMs<u8> + DelayUs<u16>,
{
    fn delay_us(&mut self, us: u16) {
        let mut d = self.delay.try_borrow_mut().unwrap(); //Fail loudly
        d.deref_mut().delay_us(us)
    }
}

impl<D> DelayMs<u8> for SharedDelayHandler<'_, D>
where
    D: DelayMs<u8> + DelayUs<u16>,
{
    fn delay_ms(&mut self, ms: u8) {
        let mut d = self.delay.try_borrow_mut().unwrap(); //Fail loudly
        d.deref_mut().delay_ms(ms)
    }
}

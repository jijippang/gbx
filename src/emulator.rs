
use crate::consoles::Console;



#[derive(Debug)]
pub struct Emulator<C: Console>
{
    pub console: C,
}


impl<C: Console> Emulator<C>
{
    pub fn new(console: C) -> Self
    {
        Self {
            console: console,
        }
    }

    pub fn generate_frame(&mut self)
    {
        self.console.step(1000);
    }
}


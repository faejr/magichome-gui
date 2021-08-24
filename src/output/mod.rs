use std::str::FromStr;

pub mod magic_home;

#[derive(Debug, PartialEq)]
pub enum State {
    ON,
    OFF
}

pub enum Mode {
    FADE,
    INSTANT
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(input: &str) -> Result<Mode, Self::Err> {
        match input.to_lowercase().as_ref() {
            "fade"  => Ok(Mode::FADE),
            "instant"  => Ok(Mode::INSTANT),
            _      => Err(()),
        }
    }
}

pub trait Output {
    fn set_mode(&mut self, mode: Mode);
    fn connect(&mut self) -> anyhow::Result<()>;
    fn is_connected(&self) -> bool;
    fn set_color(&mut self, rgb: [u8; 3]) -> anyhow::Result<()>;
    fn on_off(&mut self) -> anyhow::Result<()>;
}

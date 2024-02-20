#![cfg_attr(not(test), no_std)]

pub mod fsm;

pub enum JTAGError {
}

pub type Result<T> = core::result::Result<T, JTAGError>;

pub trait Interface {
    fn walk(&mut self, tms: &[bool]) -> Result<()>;
    fn exchange(&mut self, tdi: &mut [bool]) -> Result<()>;
}

pub struct Adapter<I: Interface> {
    iface: I,
    state: Option<fsm::State>,
}

impl <I: Interface> Adapter<I> {
    pub fn new(iface: I) -> Self {
        Self {
            iface,
            state: None,
        }
    }

    pub fn walk(&mut self, to: fsm::State) -> Result<()> {
        if self.state.is_none() {
            self.iface.walk(&[true, true, true, true, true])?;
            self.state = Some(fsm::State::TestLogicReset);
        }

        let from = self.state.unwrap();
        let steps = fsm::walk_from_to(from, to);
        self.iface.walk(steps.as_slice())
    }
}


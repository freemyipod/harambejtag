use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    TestLogicReset,
    RunTestIdle,
    SelectDRScan,
    CaptureDR,
    ShiftDR,
    Exit1DR,
    PauseDR,
    Exit2DR,
    UpdateDR,
    SelectIRScan,
    CaptureIR,
    ShiftIR,
    Exit1IR,
    PauseIR,
    Exit2IR,
    UpdateIR,
}

// From, (0, 1)
const FSM_EDGES: [(State, (State, State)); 16] = [
    (State::TestLogicReset, (State::RunTestIdle, State::TestLogicReset)),
    (State::RunTestIdle, (State::RunTestIdle, State::SelectDRScan)),
    (State::SelectDRScan, (State::CaptureDR, State::SelectIRScan)),
    (State::SelectIRScan, (State::CaptureIR, State::TestLogicReset)),

    (State::CaptureDR, (State::ShiftDR, State::Exit1DR)),
    (State::ShiftDR, (State::ShiftDR, State::Exit1DR)),
    (State::Exit1DR, (State::PauseDR, State::UpdateDR)),
    (State::PauseDR, (State::PauseDR, State::Exit2DR)),
    (State::Exit2DR, (State::ShiftDR, State::UpdateDR)),
    (State::UpdateDR, (State::RunTestIdle, State::SelectDRScan)),

    (State::CaptureIR, (State::ShiftIR, State::Exit1IR)),
    (State::ShiftIR, (State::ShiftIR, State::Exit1IR)),
    (State::Exit1IR, (State::PauseIR, State::UpdateIR)),
    (State::PauseIR, (State::PauseIR, State::Exit2IR)),
    (State::Exit2IR, (State::ShiftIR, State::UpdateIR)),
    (State::UpdateIR, (State::RunTestIdle, State::SelectDRScan)),
];

fn edges_from(from: State) -> (State, State) {
    for (f, t) in FSM_EDGES.iter() {
        if from == *f  {
            return t.clone()
        }
    }
    panic!("Unknown state {:?}", from)
}

pub fn walk_from_to(from: State, to: State) -> Vec::<bool, 8> {
    if from == to {
        return Vec::new();
    }

    let mut walked = Vec::<(State, State, bool), 16>::new();
    let mut seen = Vec::<State, 16>::new();

    let mut dequeue = Vec::<State, 5>::new();
    dequeue.push(from).unwrap();

    while dequeue.len() > 0 {
        let el = dequeue[0];
        if el == to {
            return rebuild_path(from, to, walked);
        }
        dequeue.rotate_left(1);
        dequeue.truncate(dequeue.len()-1);

        let (zero, one) = edges_from(el);
        for (b, &next) in [zero, one].iter().enumerate() {
            if seen.contains(&next) {
                continue
            }
            let edge = (el, next, b == 1);
            if let Some((ix,_)) = walked.iter().enumerate().filter(|(_, e)| e.1 == next).next() {
                walked[ix] = edge;
            } else {
                walked.push(edge).unwrap();
            }
            seen.push(next).unwrap();
            dequeue.push(next).unwrap();
        }
    }
    unreachable!();
}

fn rebuild_path(
    from: State,
    to: State,
    edges: Vec::<(State, State, bool), 16>,
)  -> Vec::<bool, 8> {
    let mut cur = to;
    let mut res = Vec::new();
    loop {
        let &(from2, _, bv) = edges.iter().filter(|e| e.1 == cur).next().unwrap();
        cur = from2;
        res.push(bv).unwrap();
        if cur == from  {
            res.reverse();
            return res;
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Make sure that we can get from any state to another state.
    fn walk_all() {
        let all = [
            State::TestLogicReset,
            State::RunTestIdle,
            State::SelectDRScan,
            State::CaptureDR,
            State::ShiftDR,
            State::Exit1DR,
            State::PauseDR,
            State::Exit2DR,
            State::UpdateDR,
            State::SelectIRScan,
            State::CaptureIR,
            State::ShiftIR,
            State::Exit1IR,
            State::PauseIR,
            State::Exit2IR,
            State::UpdateIR,
        ];
        for from in all.iter().cloned() {
            for to in all.iter().cloned() {
                walk_from_to(from, to);
            }
        }
    }
}
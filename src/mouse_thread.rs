use std::sync::mpsc::SendError;

use evdev::{AbsoluteAxisType, Device, InputEventKind, RelativeAxisType};
use futures_util::StreamExt;

#[derive(Debug)]
pub enum Lr {
    Left,
    Right,
}

#[derive(Debug)]
pub enum Ax {
    X,
    Y,
}

#[derive(Debug)]
pub struct MouseMoveData {
    pub lr: Lr,
    pub ax: Ax,
    pub v: i32,
}

pub type RelMouseMove = MouseMoveData;
pub type AbsMouseMove = MouseMoveData;

#[derive(Debug)]
pub enum MouseMove {
    Rel(RelMouseMove),
    Abs(AbsMouseMove),
}

#[tokio::main(flavor = "current_thread")]
pub async fn mouse_thread(
    tx: std::sync::mpsc::Sender<MouseMove>,
    l_dev: Device,
    r_dev: Device,
) -> Result<(), SendError<MouseMove>> {
    let l_stream = l_dev.into_event_stream().unwrap().map(|ev| (false, ev));
    let r_stream = r_dev.into_event_stream().unwrap().map(|ev| (true, ev));
    let mut events = futures_util::stream::select(l_stream, r_stream);
    while let Some((right, Ok(ev))) = events.next().await {
        let mouse_move = {
            let lr = if right { Lr::Right } else { Lr::Left };
            let v = ev.value();
            match ev.kind() {
                InputEventKind::RelAxis(ax_) => {
                    let ax = match ax_ {
                        RelativeAxisType::REL_X => Ax::X,
                        RelativeAxisType::REL_Y => Ax::Y,
                        _ => continue,
                    };
                    MouseMove::Rel(MouseMoveData { lr, ax, v })
                }
                InputEventKind::AbsAxis(ax_) => {
                    let ax = match ax_ {
                        AbsoluteAxisType::ABS_X => Ax::X,
                        AbsoluteAxisType::ABS_Y => Ax::Y,
                        _ => continue,
                    };
                    MouseMove::Rel(MouseMoveData { lr, ax, v })
                }
                _ => continue,
            }
        };
        tx.send(mouse_move)?;
    }
    Ok(())
}

use std::sync::mpsc::SendError;

use evdev::{AbsoluteAxisType, Device, InputEventKind, RelativeAxisType};
use futures_util::StreamExt;

#[derive(Debug)]
pub enum Lr {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AxType {
    Rel,
    Abs,
}

#[derive(Debug)]
pub enum Ax {
    X,
    Y,
}

#[derive(Debug)]
pub struct MouseMove {
    pub lr: Lr,
    pub ax_type: AxType,
    pub ax: Ax,
    pub v: i32,
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
        let lr = if right { Lr::Right } else { Lr::Left };
        let mouse_move = match ev.kind() {
            InputEventKind::RelAxis(ax_type) => match ax_type {
                RelativeAxisType::REL_X => MouseMove {
                    lr,
                    ax_type: AxType::Rel,
                    ax: Ax::X,
                    v: ev.value(),
                },
                RelativeAxisType::REL_Y => MouseMove {
                    lr,
                    ax_type: AxType::Rel,
                    ax: Ax::Y,
                    v: ev.value(),
                },
                _ => continue,
            },
            InputEventKind::AbsAxis(ax_type) => match ax_type {
                AbsoluteAxisType::ABS_X => MouseMove {
                    lr,
                    ax_type: AxType::Abs,
                    ax: Ax::X,
                    v: ev.value(),
                },
                AbsoluteAxisType::ABS_Y => MouseMove {
                    lr,
                    ax_type: AxType::Abs,
                    ax: Ax::Y,
                    v: ev.value(),
                },
                _ => continue,
            },
            _ => continue,
        };
        tx.send(mouse_move)?;
    }
    Ok(())
}

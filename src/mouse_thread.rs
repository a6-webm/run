use std::sync::mpsc::SendError;

use evdev::{Device, EventType, InputEventKind, RelativeAxisType};
use futures_util::StreamExt;

pub enum MouseMove {
    LeftX(i32),
    LeftY(i32),
    RightX(i32),
    RightY(i32),
}

pub async fn mouse_thread(
    tx: std::sync::mpsc::Sender<MouseMove>,
    l_dev: Device,
    r_dev: Device,
) -> Result<(), SendError<MouseMove>> {
    let l_stream = l_dev.into_event_stream().unwrap().map(|ev| (false, ev));
    let r_stream = r_dev.into_event_stream().unwrap().map(|ev| (true, ev));
    let mut events = futures_util::stream::select(l_stream, r_stream);
    while let Some((right, Ok(ev))) = events.next().await {
        if ev.event_type() != EventType::RELATIVE {
            continue;
        }
        match (ev.kind(), right) {
            (InputEventKind::RelAxis(RelativeAxisType::REL_X), false) => {
                tx.send(MouseMove::LeftX(ev.value()))?;
            }
            (InputEventKind::RelAxis(RelativeAxisType::REL_Y), false) => {
                tx.send(MouseMove::LeftY(ev.value()))?;
            }
            (InputEventKind::RelAxis(RelativeAxisType::REL_X), true) => {
                tx.send(MouseMove::RightX(ev.value()))?;
            }
            (InputEventKind::RelAxis(RelativeAxisType::REL_Y), true) => {
                tx.send(MouseMove::RightY(ev.value()))?;
            }
            _ => (),
        }
    }
    Ok(())
}

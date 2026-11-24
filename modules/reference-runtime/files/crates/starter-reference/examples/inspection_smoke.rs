use eframe::egui::{
    Event,
    accesskit::{Action, ActionRequest, TreeUpdate},
};
use egui_inspection::{Request, Response, protocol::read_handshake, read_message, write_message};
use std::{
    net::TcpStream,
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args_os().nth(1).ok_or("capture path required")?);
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut stream = loop {
        match TcpStream::connect(egui_inspection::DEFAULT_INSPECTION_ADDR) {
            Ok(stream) => break stream,
            Err(error) if Instant::now() >= deadline => return Err(error.into()),
            Err(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    assert_eq!(
        read_handshake(&mut stream)?,
        egui_inspection::PROTOCOL_VERSION
    );
    let mut call = |request| -> std::io::Result<Response> {
        write_message(&mut stream, &request)?;
        read_message(&mut stream)
    };
    let before = expect_tree(call(Request::GetTree)?)?;
    let (id, _) = before
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("New note"))
        .ok_or("New note button missing")?;
    let event = Event::AccessKitActionRequest(ActionRequest {
        action: Action::Click,
        target_tree: before.tree_id,
        target_node: *id,
        data: None,
    });
    assert!(matches!(
        call(Request::ApplyEvents {
            events: vec![event]
        })?,
        Response::Done
    ));
    let after = expect_tree(call(Request::GetTree)?)?;
    assert!(
        after
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Note 2")),
        "note creation was not visible"
    );
    let Response::Screenshot(image) = call(Request::GetScreenshot {
        pixels_per_point: None,
    })?
    else {
        return Err("inspection did not return a screenshot".into());
    };
    assert!(image.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    std::fs::write(output, image.bytes)?;
    Ok(())
}
fn expect_tree(response: Response) -> Result<TreeUpdate, Box<dyn std::error::Error>> {
    if let Response::Tree {
        accesskit: Some(tree),
        ..
    } = response
    {
        Ok(tree)
    } else {
        Err(format!("missing AccessKit tree: {response:?}").into())
    }
}

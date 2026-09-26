//! The client's messages waiting to be handled.
//!
//! A thread of its own reads stdin, so the messages the client sends while the server works on one
//! request queue up here, and a cancellation reaches the request it names while that request is
//! still queued. A cancelled request is answered with `RequestCancelled` at once, instead of being
//! carried out. A cancellation of a request already handed out goes to the server, which answers
//! that request the same way when it is still holding it unanswered. An editor sends a completion
//! request with every keystroke and cancels the one before it; on a slow machine carrying out each
//! completion takes longer than the keystrokes take to arrive, and answering the cancelled ones at
//! once lets the server keep up with the typing.

use super::server::{parse_params, JSONRPCMessage};
use crate::write_log;
use lsp_types::{CancelParams, NumberOrString};
use std::collections::VecDeque;
use std::io::{stdin, BufRead};
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// What the server is handed out of the inbox.
pub(super) enum Incoming {
    /// A message to handle.
    Message(JSONRPCMessage),
    /// The id of a request the client cancelled before the server reached it, which is answered
    /// with `RequestCancelled`.
    Cancelled(u32),
    /// The id a cancellation names when the request is no longer queued: the server has handed it
    /// out already, and may be holding it unanswered.
    LateCancellation(u32),
}

/// The messages the client has sent and the server has still to handle, in the order the client
/// sent them.
pub(super) struct Inbox {
    /// The messages the reader thread has read off stdin. It disconnects once stdin is closed.
    recv: Receiver<JSONRPCMessage>,
    /// The messages taken off `recv` and not yet handed out by `next`.
    queued: VecDeque<Incoming>,
}

impl Inbox {
    /// Start the thread that reads the client's messages off stdin, and return the inbox they
    /// arrive in.
    pub(super) fn spawn_stdin_reader() -> Inbox {
        let (send, recv) = mpsc::channel();
        thread::Builder::new()
            .name("lsp-stdin".to_string())
            .spawn(move || {
                let mut stdin = stdin().lock();
                while let Some(message) = read_message(&mut stdin) {
                    if send.send(message).is_err() {
                        break;
                    }
                }
            })
            .expect("failed to spawn the thread reading stdin");
        Inbox {
            recv,
            queued: VecDeque::new(),
        }
    }

    /// The next message to handle, waiting for one to arrive when none has. `None` once stdin is
    /// closed and every message read off it has been handed out.
    ///
    /// Every message that has arrived is queued before the next is handed out, so that a
    /// cancellation the client has already sent reaches the request it names.
    pub(super) fn next(&mut self) -> Option<Incoming> {
        loop {
            self.take_arrived();
            if let Some(incoming) = self.queued.pop_front() {
                return Some(incoming);
            }
            let message = self.recv.recv().ok()?;
            self.accept(message);
        }
    }

    /// Queue every message that has arrived, without waiting for more.
    fn take_arrived(&mut self) {
        while let Ok(message) = self.recv.try_recv() {
            self.accept(message);
        }
    }

    /// Queue `message`. A cancellation marks the queued request it names as cancelled instead, or
    /// queues a `LateCancellation` when that request has been handed out.
    fn accept(&mut self, message: JSONRPCMessage) {
        if message.method.as_deref() != Some("$/cancelRequest") {
            self.queued.push_back(Incoming::Message(message));
            return;
        }
        let params = message.params.and_then(parse_params::<CancelParams>);
        let Some(CancelParams {
            id: NumberOrString::Number(id),
        }) = params
        else {
            return;
        };
        let Ok(id) = u32::try_from(id) else {
            return;
        };
        let request = self.queued.iter_mut().find(|incoming| {
            matches!(incoming, Incoming::Message(queued_message)
                if queued_message.method.is_some() && queued_message.id == Some(id))
        });
        match request {
            Some(request) => *request = Incoming::Cancelled(id),
            None => self.queued.push_back(Incoming::LateCancellation(id)),
        }
    }
}

/// Read one message off `stdin`, skipping what does not read as a message. `None` once stdin is
/// closed.
fn read_message(stdin: &mut impl BufRead) -> Option<JSONRPCMessage> {
    loop {
        // Read a line to get the content length.
        let mut header_line = String::new();
        match stdin.read_line(&mut header_line) {
            // `read_line` returns `Ok(0)` when stdin has reached EOF, which happens when the
            // parent editor process dies and closes the pipe. EOF is permanent: every subsequent
            // read returns `Ok(0)` immediately without blocking, so reading on would spin at 100%
            // CPU forever.
            Ok(0) => {
                write_log!("stdin reached EOF. Exiting the language server.");
                return None;
            }
            Ok(_) => {}
            Err(e) => {
                write_log!("Failed to read a line: \n{:?}", e);
                continue;
            }
        }
        if header_line.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !header_line.starts_with("Content-Length:") {
            write_log!(
                "Expected `Content-Length:`. The line is: \n{:?}",
                header_line
            );
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: usize = match header_line["Content-Length:".len()..].trim().parse() {
            Ok(content_length) => content_length,
            Err(e) => {
                write_log!("Failed to parse the content length: \n{:?}", e);
                continue;
            }
        };

        // Read stdin up to an empty line.
        loop {
            let mut line = String::new();
            if let Err(e) = stdin.read_line(&mut line) {
                write_log!("Failed to read a line: \n{:?}", e);
                continue;
            }
            if line.trim().is_empty() {
                break;
            }
        }

        // Read the content of the message.
        let mut message = vec![0; content_length];
        if let Err(e) = stdin.read_exact(&mut message) {
            write_log!("Failed to read the message: \n{:?}", e);
            continue;
        }
        let message = match String::from_utf8(message) {
            Ok(message) => message,
            Err(e) => {
                write_log!("Failed to parse the message as utf-8 string: \n{:?}", e);
                continue;
            }
        };

        // Parse the message as JSONRPCMessage.
        match serde_json::from_str::<JSONRPCMessage>(&message) {
            Ok(message) => {
                write_log!(
                    "Received message: {:?}",
                    serde_json::to_string(&message).unwrap()
                );
                return Some(message);
            }
            Err(e) => {
                write_log!("Failed to parse the message as JSONRPCMessage: \n{:?}", e);
                continue;
            }
        }
    }
}

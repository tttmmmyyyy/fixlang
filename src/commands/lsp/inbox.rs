//! The client's messages waiting to be handled.
//!
//! A thread of its own reads stdin, so the messages the client sends while the server works on one
//! request queue up here. Before the server carries out a request, the queue behind it says whether
//! the answer still has a use: the client may have cancelled it, or changed the document it asks
//! about. Such a request is answered with an error at once. An editor sends a completion request
//! and a semantic-tokens request with every keystroke, and on a slow machine carrying each of them
//! out takes longer than the keystrokes take to arrive; answering the outdated ones at once keeps
//! the server level with the typing.

use super::server::{JSONRPCMessage, ResponseError};
use crate::misc::Set;
use crate::write_log;
use lsp_types::{CancelParams, NumberOrString};
use std::collections::VecDeque;
use std::io::{stdin, BufRead};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

/// The messages the client has sent and the server has still to handle, in the order the client
/// sent them.
pub(super) struct Inbox {
    /// The messages the reader thread has read off stdin. It disconnects once stdin is closed.
    recv: Receiver<JSONRPCMessage>,
    /// The messages taken off `recv` and not yet handed out by `next`.
    queued: VecDeque<JSONRPCMessage>,
    /// The ids of the requests in `queued` that the client has cancelled.
    cancelled: Set<u32>,
}

impl Inbox {
    /// Start the thread that reads the client's messages off stdin, and return the inbox they
    /// arrive in.
    pub(super) fn read_stdin() -> Inbox {
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
            cancelled: Set::default(),
        }
    }

    /// The next message to handle, waiting for one to arrive when none has. `None` once stdin is
    /// closed and every message read off it has been handed out.
    ///
    /// Every message that has arrived is queued before the next is handed out, so that
    /// `obsolete` judges it against everything the client has sent so far.
    pub(super) fn next(&mut self) -> Option<JSONRPCMessage> {
        self.take_arrived();
        if self.queued.is_empty() {
            let message = self.recv.recv().ok()?;
            self.enqueue(message);
            self.take_arrived();
        }
        self.queued.pop_front()
    }

    /// The error to answer `request` with in place of carrying it out, when the messages queued
    /// behind it leave its answer with no use: the client cancelled it, or changed the document it
    /// asks about. `request` is the message `next` has just handed out.
    pub(super) fn obsolete(&mut self, request: &JSONRPCMessage) -> Option<ResponseError> {
        let id = request.id?;
        if request.method.is_none() {
            return None;
        }
        if self.cancelled.remove(&id) {
            return Some(ResponseError::request_cancelled());
        }
        let uri = document_of(request)?;
        let changed_later = self.queued.iter().any(|message| {
            message.method.as_deref() == Some("textDocument/didChange")
                && document_of(message) == Some(uri)
        });
        changed_later.then(ResponseError::content_modified)
    }

    /// Queue every message that has arrived, without waiting for more.
    fn take_arrived(&mut self) {
        loop {
            match self.recv.try_recv() {
                Ok(message) => self.enqueue(message),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return,
            }
        }
    }

    /// Queue `message`. A cancellation is recorded against the request it names instead, when
    /// that request is still queued; one naming a request already handed out has nothing left to
    /// act on.
    fn enqueue(&mut self, message: JSONRPCMessage) {
        if message.method.as_deref() != Some("$/cancelRequest") {
            self.queued.push_back(message);
            return;
        }
        let params = message
            .params
            .and_then(|params| serde_json::from_value::<CancelParams>(params).ok());
        let Some(CancelParams {
            id: NumberOrString::Number(id),
        }) = params
        else {
            return;
        };
        let Ok(id) = u32::try_from(id) else {
            return;
        };
        if self.queued.iter().any(|message| message.id == Some(id)) {
            self.cancelled.insert(id);
        }
    }
}

/// The URI of the document a message asks about or reports on, which is `textDocument.uri` among
/// its params.
fn document_of(message: &JSONRPCMessage) -> Option<&str> {
    message
        .params
        .as_ref()?
        .pointer("/textDocument/uri")?
        .as_str()
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

        // Read stdin upto an empty line.
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

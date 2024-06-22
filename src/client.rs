//MIT License

//Copyright (c) 2017 Colin Rothfels

//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:

//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.

//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

use std::convert::TryInto;
use std::sync::mpsc::channel;
use std::sync::{Mutex, Arc};
use std::thread;
use std::process::{ChildStdin, Child};
use std::io::{Write, BufReader};
use std::collections::HashMap;

use serde_json::Value;
use serde_json;
use jsonrpc_lite::{Error, Id, JsonRpc};

use crate::parsing;

// this to get around some type system pain related to callbacks. See:
// https://doc.rust-lang.org/beta/book/trait-objects.html,
// http://stackoverflow.com/questions/41081240/idiomatic-callbacks-in-rust
trait Callable: Send {
    fn call(self: Box<Self>, result: Result<Value, Value>);
}

impl<F:Send + FnOnce(Result<Value, Value>)> Callable for F {
    fn call(self: Box<F>, result: Result<Value, Value>) {
        (*self)(result)
    }
}

type Callback = Box<dyn Callable>;

// add a callback
pub struct Event {
    handlers:  Vec<Box<dyn Fn(&Value) + Send >>,
}

impl Event {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }
    
    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(&Value) + 'static +  Send,
    {
        self.handlers.push(Box::new(handler));
    }
    pub fn emit(&mut self, payload: &Value) {
        for handler in &self.handlers {
            handler(payload);
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Self { handlers: Default::default() }
    }
}

/// Represents (and mediates communcation with) a Language Server.
///
/// LanguageServer should only ever be instantiated or accessed through an instance of
/// LanguageServerRef, which mediates access to a single shared LanguageServer through a Mutex.
// #[derive(Default)]
struct LanguageServer<W: Write> {
    peer: W,
    pending: HashMap<usize, Callback>,
    next_id: usize,
    event: Event,
    // listeners: HashMap<usize, Callback>,
    // listener_id: usize,
}


/// Generates a Language Server Protocol compliant message.
fn prepare_lsp_json(msg: &Value) -> Result<String, serde_json::error::Error> {
    let request = serde_json::to_string(&msg)?;
    Ok(format!("Content-Length: {}\r\n\r\n{}", request.len(), request))
}

impl <W:Write> LanguageServer<W> {
    fn write(&mut self, msg: &str) {
        self.peer.write_all(msg.as_bytes()).expect("error writing to stdin");
        self.peer.flush().expect("error flushing child stdin");
    }

    fn send_request(&mut self, method: &str, params: &Value, completion: Callback) {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params
        });

        self.pending.insert(self.next_id, completion);
        self.next_id += 1;
        self.send_rpc(&request);
    }

    pub fn add_listener<F>(&mut self, completion: F) 
    where
        F: Fn(&Value) + 'static +  Send,
    {
        self.event.subscribe(completion);
    }

    fn send_notification(&mut self, method: &str, params: &Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.send_rpc(&notification);
    }

    fn handle_response(&mut self, id: usize, result: Value) {
        let callback = self.pending.remove(&id).expect(&format!("id {} missing from request table", id));
        callback.call(Ok(result));
    }

    fn handle_notification(&mut self, result: &Value) {
        self.event.emit(result);
    }

    fn handle_error(&mut self, id: usize, error: Value) {
        let callback = self.pending.remove(&id).expect(&format!("id {} missing from request table", id));
        callback.call(Err(error));
    }

    fn send_rpc(&mut self, rpc: &Value) {
        let rpc = match prepare_lsp_json(&rpc) {
            Ok(r) => r,
            Err(err) => panic!("error encoding rpc {:?}", err),
        };
        self.write(&rpc);
    }
}

/// Access control and convenience wrapper around a shared LanguageServer instance.
pub struct LanguageServerRef<W: Write>(Arc<Mutex<LanguageServer<W>>>);

// //FIXME: this is hacky, and prevents good error propogation,
// fn number_from_id(id: Option<&Value>) -> usize {
//     let id = id.expect("response missing id field");
//     let id = match id {
//         &Value::Number(ref n) => n.as_u64().expect("failed to take id as u64"),
//         &Value::String(ref s) => u64::from_str_radix(s, 10).expect("failed to convert string id to u64"),
//         other => panic!("unexpected value for id field: {:?}", other),
//     };

//     id as usize
// }

impl<W: Write> LanguageServerRef<W> {
    fn new(peer: W) -> Self {
        LanguageServerRef(Arc::new(Mutex::new(LanguageServer {
            peer,
            pending: HashMap::new(),
            next_id: 1,
            event: Event::default(),
            // listeners: HashMap::new(),
            // listener_id: 1,
        })))
    }

    /// Writes `msg` to the underlying process's stdin. Exposed for testing & debugging; 
    /// you should not need to call this method directly.
    fn write(&self, msg: &str) {
        let mut inner = self.0.lock().unwrap();
        inner.write(msg);
    }

    //TODO: real logging (with slog?)
    fn handle_msg(&self, val: &Value) {
        
        let parsed_value = JsonRpc::parse(val.to_string().as_str());
        if let Err(err) = parsed_value {
            // println!("error parsing json: {:?}", err);
            return;
        }
        let parsed_value = parsed_value.expect("to be present");

        match parsed_value.clone() {
            JsonRpc::Request(_) => {
                // println!("request");
                // println!("request=====: \n {:#?}\n", parsed_value);
            },
            JsonRpc::Notification(_) => {
                // println!("Notification");
                // println!("Notification=====: \n {:#?}\n", parsed_value);
                // let method = parsed_value.get_method().unwrap();
                // let params = val.clone();

                let mut inner = self.0.lock().unwrap();
                inner.handle_notification(&val);
            },
            JsonRpc::Success(_) => {
                // println!("Success=====: \n {:#?}\n", parsed_value);
                let id = parsed_value.get_id();
                let response = parsed_value.get_result();
                let error = parsed_value.get_error();
                match (id, response, error) {
                    (Some(Id::Num(id)), Some(response), None) => {
                        let mut inner = self.0.lock().unwrap();
                        inner.handle_response(id.try_into().unwrap(), response.clone());
                    }
                    (Some(Id::Num(id)), None, Some(error)) => {
                        let mut inner = self.0.lock().unwrap();
                        inner.handle_error(id.try_into().unwrap(), Value::String(error.message.clone()));
                    }
                    (Some(Id::Num(id)), Some(response), Some(error)) => {
                        panic!("We got both response and error.. what even??");
                    }
                    _ => {}
                }
            },
            JsonRpc::Error(_) => {
                // println!("Error");
            },
        };


    }

    // listening notification messages (JSON-RPC ) from the Language Server
    pub fn add_listener<F>(&mut self, completion: F) 
    where
        F: Fn(&Value) + 'static +  Send 
    {
        let mut inner = self.0.lock().unwrap();
        inner.add_listener(completion)
    }

    /// Sends a JSON-RPC request message with the provided method and parameters.
    /// `completion` should be a callback which will be executed with the server's response.
    pub fn send_request<CB>(&self, method: &str, params: &Value, completion: CB)
        where CB: 'static + Send + FnOnce(Result<Value, Value>) {
            let mut inner = self.0.lock().unwrap();
            inner.send_request(method, params, Box::new(completion));
    }

    /// Sends a JSON-RPC notification message with the provided method and parameters.
    pub fn send_notification(&self, method: &str, params: &Value) {
        let mut inner = self.0.lock().unwrap();
        inner.send_notification(method, params);
    }
}

impl <W: Write> Clone for LanguageServerRef<W> {
    fn clone(&self) -> Self {
        LanguageServerRef(self.0.clone())
    }
}

pub fn start_language_server(mut child: Child) -> (Child, LanguageServerRef<ChildStdin>) {
    let child_stdin = child.stdin.take().unwrap();
    let child_stdout = child.stdout.take().unwrap();
    let lang_server = LanguageServerRef::new(child_stdin);
    {
        let lang_server = lang_server.clone();
        thread::spawn(move ||{
            let mut reader = BufReader::new(child_stdout);
            loop {
                match parsing::read_message(&mut reader) {
                    Ok(ref val) => lang_server.handle_msg(val),
                    Err(err) => println!("parse error: {:?}", err),
                };
            }
        });
    }
    (child, lang_server)
}



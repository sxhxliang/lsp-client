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

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate lsp_client;

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use std::process::{Child, ChildStdin, Command, Stdio};
use lsp_client::{start_language_server, LanguageServerRef};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub struct GPTEngine {
    pub lsp_lang_server: Option<Arc<Mutex<LanguageServerRef<ChildStdin>>>>,
}


#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Composition {
    pub length: usize,
    pub cursor_pos: usize,
    pub sel_start: usize,
    pub sel_end: usize,
    pub preedit: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Menu {
    pub page_size: usize,
    pub page_no: usize,
    pub is_last_page: bool,
    pub highlighted_candidate_index: usize,
    pub num_candidates: usize,
    pub candidates: Vec<Candidate>,
    pub select_keys: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Candidate {
    pub text: String,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct RequestParams {
    pub process_id: String,
    pub key_sequence: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct ResponseData {
    pub method: String,
    pub params: RequestParams,
    pub commit: Option<String>,
    composition: Option<Composition>,
    pub menu: Option<Menu>,
    pub message: Option<String>
}


fn main() {
    println!("starting main read loop");
    let (mut child, mut lang_server) = start_language_server(prepare_command());


    // this init blob was copied from the atom client example here:
    // https://github.com/jonathandturner/rls_vscode/blob/master/src/extension.ts
    // let init = json!({
    //     "process_id": "Null",
    //     "root_path": "/Users/cmyr/Dev/hacking/xi-mac/xi-editor", // a path to some rust project
    //     "initialization_options": {},
    //     "capabilities": {
    //         "documentSelector": ["rust"],
    //         "synchronize": {
    //             "configurationSection": "languageServerExample"
    //         }
    //     },
    // });

    let (sender, receiver) = channel();
    // thread::spawn(move || {
    //     let result = 42;
    //     sender.send(result).unwrap();
    // });

    let init = json!({
        "process_id": "Null",
        "key_sequence": "woshiyigeren", // a path to some rust project
    });

    let _sender = sender.clone();

    // let counter = Arc::new(Mutex::new(lang_server));

    // let engine = GPTEngine{
    //     lsp_lang_server: Some(counter),
    // };
    // let binding = engine.lsp_lang_server.unwrap();
    // let lang_server = binding.lock().unwrap();


    lang_server.add_listener(|key| {
        println!("Pressed1 \"{}\"", key);
    });
    lang_server.add_listener(|key| {
        println!("Pressed2 \"{}\"", key);
    });
    lang_server.add_listener(|key| {
        println!("Pressed3 \"{}\"", key);
    });
    
    lang_server.send_request("simulate_key_sequence", &init, move |result| {
        // println!("received response {:?}", result);
        _sender.send(result).unwrap();
        
    });
    let result = receiver.recv().unwrap();
    println!("Result: {:#?}", &result);

    // let params: ResponseData = serde_json::from_value(result.unwrap()).unwrap();
    // println!("Params: {:#?}", &params);

    // let init = json!({
    //     "process_id": "Null",
    //     "key_sequence": "chatgpt", // a path to some rust project
    // });

    // let _sender: std::sync::mpsc::Sender<Result<serde_json::Value, serde_json::Value>> = sender.clone();
    // lang_server.send_request("command", &init, move |result| {
    //     println!("received response {:?}", result);
    //     _sender.send(result).unwrap();
        
    // });
    // let result = receiver.recv().unwrap();

    // let init = json!({
    //     "process_id": "Null",
    //     "key_sequence": "nihaoa ", // a path to some rust project
    // });
    // let _sender: std::sync::mpsc::Sender<Result<serde_json::Value, serde_json::Value>> = sender.clone();
    // lang_server.send_request("simulate_key_sequence", &init, move |result| {
    //     println!("received response {:?}", result);
    //     _sender.send(result).unwrap();
        
    // });
    // let result = receiver.recv().unwrap();
    
    // let init = json!({
    //     "process_id": "Null",
    //     "key_sequence": "?", // a path to some rust project
    // });
    // let _sender: std::sync::mpsc::Sender<Result<serde_json::Value, serde_json::Value>> = sender.clone();
    // lang_server.send_request("command", &init, move |result| {
    //     println!("received response {:?}", result);
    //     _sender.send(result).unwrap();
        
    // });
    // let result = receiver.recv().unwrap();


    // // let result = receiver.recv().unwrap();
    // // println!("Result: {:?}", result);

    // let init = json!({
    //     "process_id": "Null",
    //     "key_sequence": "ceshiyixia ", // a path to some rust project
    // });

    // let _sender: std::sync::mpsc::Sender<Result<serde_json::Value, serde_json::Value>> = sender.clone();
    // lang_server.send_request("simulate_key_sequence", &init, move |result| {
    //     // println!("received response {:?}", result);
    //     _sender.send(result).unwrap();
        
    // });
    // let result = receiver.recv().unwrap();
    // println!("Result: {:?}", result);

    // let init = json!({
    //     "process_id": "Null",
    //     "key_sequence": "?", // a path to some rust project
    // });

    // let _sender: std::sync::mpsc::Sender<Result<serde_json::Value, serde_json::Value>> = sender.clone();
    // lang_server.send_request("command", &init, move |result| {
    //     // println!("received response {:?}", result);
    //     _sender.send(result).unwrap();
        
    // });
    // let result = receiver.recv().unwrap();
    // println!("Result: {:?}", result);

    // lang_server.send_request("initialize", &init, |result| {
    //     println!("received response {:?}", result);
    // });
    // lang_server.send_request("initialize", &init, |result| {
    //     println!("received response {:?}", result);
    // });
    // lang_server.send_request("initialize", &init, |result| {
    //     println!("received response {:?}", result);
    // });

    child.wait();
}

fn prepare_command() -> Child {
    // use std::env;
    // let rls_root = env::var("RLS_ROOT").expect("$RLS_ROOT must be set");
    // target\x86_64-pc-windows-msvc\debug\wensi-lsp.exe
    // Command::new("..\\..\\rime-api\\target\\x86_64-pc-windows-msvc\\debug\\rime-api.exe")
        // .args(&["--listen"])
    // Command::new(".\\stdio.exe")
    // let mut child = Command::new(".\\wensi-lsp.exe");
    // let mut child = Command::new("..\\wensi-lsp\\target\\x86_64-pc-windows-msvc\\debug\\wensi-lsp.exe");
    
    let mut child = Command::new("../wensi-lsp/target/debug/wensi-lsp");
    // let mut child = Command::new("C:\\Program Files (x86)\\Wensi\\wensi-lsp.exe");
    if let Ok(s) = &child.output() {
        // if s.status.success() {
        //     println!("gpt_server availability detected via subprocess");
        // } else {
        //     println!("failed gpt_server availability detected via subprocess");
        // }
       
    }

    let child = child.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start rls");
    
    child
}


// fn prepare_command() -> Child {
//     use std::env;
//     let rls_root = env::var("RLS_ROOT").expect("$RLS_ROOT must be set");
//         Command::new("cargo")
//             .current_dir(rls_root)
//             .args(&["run", "--release"])
//             .stdin(Stdio::piped())
//             .stdout(Stdio::piped())
//             .spawn()
//             .expect("failed to start rls")
// }

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use anyhow;
use std::{
  env,
  fs::*,
  io::*,
  os::unix::net::UnixListener,
  path::PathBuf,
  process::{Command, Stdio},
  string::String,
  sync::mpsc,
  thread,
};
use tauri::api::*;
#[macro_use]
extern crate log;

mod cmd;
mod rpc;
use cmd::*;
use rpc::*;

fn get_message<R: Read>(r: &mut R) -> JSONRpcResp {
  let mut str = String::from("");
  let mut arr = [0; 256];
  loop {
    // TODO handle this error
    let res = r.read(&mut arr).expect("Read should succeed");
    if res != 0 {}
    for i in 0..res {
      str.push(arr[i] as char);
    }
    if res != 0 && arr[res - 1] == 10 {
      break;
    }
  }
  serde_json::from_str(&str).unwrap()
}
fn rxtx_server<T: Read + Write>(server: &mut T, req: JSONRpcReq) -> JSONRpcResp {
  let r = serde_json::to_string(&req).unwrap();
  trace!(
    "Server request: \"{}\"",
    serde_json::to_string_pretty(&r).unwrap()
  );
  server
    .write(&r.as_bytes())
    .expect("Failed to write to server");
  server.write(b"\n").expect("Failed to write to server");
  let r = get_message(server);
  trace!(
    "Server response: \"{}\"",
    serde_json::to_string_pretty(&r).unwrap()
  );
  return r;
}
fn server_thread<T: Read + Write>(
  mut server: T,
  rx: mpsc::Receiver<(tauri::WebviewMut, Cmd)>,
) -> () {
  loop {
    // TODO: This is horrible. DRY!
    if let Ok((mut _webview, event)) = rx.recv() {
      match event {
        Cmd::GetStatus { callback, error } => {
          match rxtx_server(&mut server, get_status()) {
            JSONRpcResp {
              result: Some(serde_json::Value::Bool(b)),
              ..
            } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(b), callback, error);
              });
            }
            JSONRpcResp { error: Some(e), .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(e), callback, error);
              });
            }
            _ => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(String::from("Unknown error")),
                  callback,
                  error,
                );
              });
            }
          };
        }
        Cmd::SetShouldRemoveNoise {
          value,
          callback,
          error,
        } => {
          // let r = serde_json::to_string(&set_should_remove_noise(value)).unwrap();
          // println!("Server request: {}", serde_json::to_string(&r).unwrap());
          // server.write(&r.as_bytes());
          // server.write(b"\n");
          // let r = get_message(& mut server);
          // println!("Server response: {}", serde_json::to_string(&r).unwrap());
          match rxtx_server(&mut server, set_should_remove_noise(value)) {
            JSONRpcResp { result: None, .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(serde_json::Value::Null),
                  callback,
                  error,
                );
              });
            }
            JSONRpcResp { error: Some(e), .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(e), callback, error);
              });
            }
            _ => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(String::from("Unknown error")),
                  callback,
                  error,
                );
              });
            }
          };
        }
        Cmd::SetLoopback {
          value,
          callback,
          error,
        } => {
          match rxtx_server(&mut server, set_loopback(value)) {
            JSONRpcResp {
              result: Some(serde_json::Value::Bool(b)),
              ..
            } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(b), callback, error);
              });
            }
            JSONRpcResp {
              result: Some(b), ..
            } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(String::from("Unknown result")),
                  callback,
                  error,
                );
              });
            }
            JSONRpcResp { error: Some(e), .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(e), callback, error);
              });
            }
            _ => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(String::from("Unknown error")),
                  callback,
                  error,
                );
              });
            }
          };
        }
        Cmd::GetMicrophones { callback, error } => {
          match rxtx_server(&mut server, get_microphones()) {
            JSONRpcResp {
              result: Some(serde_json::Value::Object(m)),
              ..
            } => {
              // Todo verify v is of correct form
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(m), callback, error);
              });
            }
            JSONRpcResp {
              result: Some(e), ..
            } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(format!("Unexpected result: {}", e)),
                  callback,
                  error,
                );
              });
            }
            JSONRpcResp { error: Some(e), .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(e), callback, error);
              });
            }
            _ => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(String::from("Unknown error")),
                  callback,
                  error,
                );
              });
            }
          };
        }
        Cmd::SetMicrophone {
          value,
          callback,
          error,
        } => {
          match rxtx_server(&mut server, set_microphones(value)) {
            JSONRpcResp { error: None, .. } => {
              // Todo verify v is of correct form
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(
                  wv,
                  move || Ok(serde_json::Value::Null),
                  callback,
                  error,
                );
              });
            }
            JSONRpcResp { error: Some(e), .. } => {
              _webview.dispatch(move |wv| {
                tauri::execute_promise_sync(wv, move || Ok(e), callback, error);
              });
            }
          };
        }
        Cmd::Exit => {
          break;
        }
        Cmd::Log { .. } => {
          error!("RPC thread received Cmd::Log message!");
        }
      }
    }
  }
}
// https://github.com/tauri-apps/tauri/issues/1308
fn get_real_resource_dir() -> Option<PathBuf> {
  let mut p = tauri::api::path::resource_dir()?;
  match env::var("APPDIR") {
    Ok(v) => {
      let mut root = PathBuf::from(v);
      if p.has_root() {
        // only on linux here
        root.push(
          p.strip_prefix("/")
            .expect("has_root() returned true; we should be able to strip / prefix"),
        );
      } else {
        root.push(p);
      }
      Some(root)
    }
    Err(_) => Some(p.into()),
  }
}

fn main() {
  env_logger::init();
  info!("Starting");
  // app is either dev or bundled. When it is dev we have to find bins
  // ourselves. Otherwise we can hopefully rely on
  // tauri_api::command::command_path
  let server_path = match env::var("SERVER_PATH") {
    Err(_) => {
      command::command_path(command::binary_command("server".to_string()).unwrap()).unwrap()
    }
    Ok(p) => p,
  };
  trace!("Server Path is: {}", server_path);

  let resource_dir = get_real_resource_dir().expect("resource dir required");

  let mut runtime_lib_path = resource_dir.clone();
  runtime_lib_path.push("native");
  runtime_lib_path.push("runtime_libs");
  info!("Setting LD_LIBRARY_PATH to {:?}", runtime_lib_path.clone().into_os_string());

  let mut sock_path = tauri::api::path::runtime_dir().expect("get runtime dir");
  sock_path.push("magic-mic.socket");

  info!("Socket path: {:?}", sock_path.clone().into_os_string());

  if metadata(sock_path.clone().into_os_string()).is_ok() {
    remove_file(sock_path.clone().into_os_string())
      .expect("Failed to remove socket path, maybe permissions?");
  }
  let listener =
    UnixListener::bind(sock_path.clone().into_os_string()).expect("Couldn't bind to unix socket");

  Command::new(server_path)
    .env("LD_LIBRARY_PATH", runtime_lib_path.into_os_string())
    .arg(sock_path.clone().into_os_string())
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("spawn server process");

  let stream = listener
    .incoming()
    .next()
    .expect("Couldn't listen on socket")
    .expect("Child failed to connect to socket");

  let (to_server, from_main) = mpsc::channel();
  thread::spawn(|| server_thread(stream, from_main));

  tauri::AppBuilder::new()
    .invoke_handler(move |_webview, arg| {
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            Cmd::Exit => Ok(()),
            Cmd::Log { msg, level } => {
              // TODO env_logger doesn't seem to print the target. not sure if I
              // am misunderstanding the purpose of target, or if env_logger
              // just doesn't do that or what, but prefixingwith "js: " is my
              // temporary workaround
              match level {
                0 => {
                  debug!(target: "js", "js: {}", msg);
                  Ok(())
                }
                1 => {
                  error!(target: "js", "js: {}", msg);
                  Ok(())
                }
                2 => {
                  info!(target: "js", "js: {}", msg);
                  Ok(())
                }
                3 => {
                  trace!(target: "js", "js: {}", msg);
                  Ok(())
                }
                4 => {
                  // 4=warn (totally intentional)
                  warn!(target: "js", "js: {}", msg);
                  Ok(())
                }
                _ => {
                  warn!(target: "js", "Recieved invalid log level from javsascript");
                  Err("Recieved invalid log level from javsascript".into())
                }
              }
            }
            c => to_server
              .send((_webview.as_mut(), c))
              .map_err(|e| format!("sending error: {}", e)),
          }
        }
      }
    })
    .build()
    .run();
  info!("Exiting");
}

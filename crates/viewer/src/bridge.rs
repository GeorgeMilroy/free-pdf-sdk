//! The viewer's side of the postMessage protocol (fps_core::protocol): it hears requests from
//! the parent page and posts replies and events back.
//!
//! Only the parent window on this page's own origin is heard (`event.origin` and
//! `event.source` are both checked) and every message must carry this instance's channel id,
//! which the wrapper puts in the iframe URL (`#ch=…`). Replies go to the parent with this origin
//! as the target, so the browser never delivers them anywhere else.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use eframe::wasm_bindgen::{JsCast, JsValue, closure::Closure};
use fps_core::protocol::{PROTOCOL_VERSION, limits};
use fps_core::{ErrorCode, RpcError};
use serde_json::Value;

/// A request as it arrived: checked envelope, unchecked parameters.
pub struct Incoming {
    pub id: u32,
    pub method: String,
    pub params: Value,
    pub data: Option<Vec<u8>>,
    /// When the wrapper stops waiting (ms since the epoch); expired requests are not run.
    pub deadline: Option<f64>,
}

/// What a reply carries.
pub enum Payload {
    Json(Value),
    Bytes(Vec<u8>),
}

pub type Inbox = Rc<RefCell<VecDeque<Incoming>>>;

/// Requests waiting for a frame; past this a request is answered `BUSY` (a page flooding the
/// viewer must not grow its memory without bound).
const MAX_QUEUED: usize = 256;

#[derive(Clone)]
pub struct Bridge {
    window: web_sys::Window,
    parent: web_sys::Window,
    origin: String,
    channel: String,
}

impl Bridge {
    /// The bridge to the parent page, or why there is none (not framed, or no channel id).
    pub fn new() -> Result<Self, String> {
        let window = web_sys::window().ok_or("no window")?;
        let parent = window.parent().ok().flatten().ok_or("the viewer must run in an iframe")?;
        if js_sys::Object::is(&parent, &window) {
            return Err("the viewer must run in an iframe (open it with FreePdf.init)".into());
        }
        let origin = window.location().origin().map_err(|_| "no origin")?;
        let hash = window.location().hash().unwrap_or_default();
        let channel = hash
            .trim_start_matches('#')
            .split('&')
            .find_map(|kv| kv.strip_prefix("ch="))
            .filter(|c| (16..=64).contains(&c.len()) && c.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or("the iframe URL has no channel id")?
            .to_owned();
        Ok(Self { window, parent, origin, channel })
    }

    fn envelope(&self, kind: &str) -> Option<js_sys::Object> {
        let m = js_sys::Object::new();
        set(&m, "fps", &JsValue::from(PROTOCOL_VERSION))?;
        set(&m, "ch", &JsValue::from_str(&self.channel))?;
        set(&m, "kind", &JsValue::from_str(kind))?;
        Some(m)
    }

    fn post(&self, message: &js_sys::Object, transfer: Option<&JsValue>) {
        let sent = match transfer {
            Some(t) => self.parent.post_message_with_transfer(message, &self.origin, &js_sys::Array::of1(t)),
            None => self.parent.post_message(message, &self.origin),
        };
        if let Err(e) = sent {
            web_sys::console::error_2(&"Free PDF SDK: postMessage failed".into(), &e);
        }
    }

    /// `hello`: the viewer is listening.
    pub fn hello(&self) {
        let Some(m) = self.envelope("hello") else { return };
        let info = serde_json::json!({ "sdk": fps_core::SDK_VERSION, "pdfcraftRev": fps_core::PDFCRAFT_REV, "protocol": PROTOCOL_VERSION });
        if set(&m, "info", &json_to_js(&info)).is_some() {
            self.post(&m, None);
        }
    }

    /// A successful reply.
    pub fn reply_ok(&self, id: u32, payload: Payload) {
        let Some(m) = self.envelope("res") else { return };
        let (result, transfer) = match payload {
            Payload::Json(v) => (json_to_js(&v), None),
            Payload::Bytes(b) => {
                let array = js_sys::Uint8Array::from(b.as_slice());
                let buffer: JsValue = array.buffer().into();
                (array.into(), Some(buffer))
            }
        };
        if set(&m, "id", &JsValue::from(id)).is_none() || set(&m, "ok", &JsValue::TRUE).is_none() || set(&m, "result", &result).is_none() {
            return;
        }
        self.post(&m, transfer.as_ref());
    }

    /// An error reply.
    pub fn reply_err(&self, id: u32, error: &RpcError) {
        let Some(m) = self.envelope("res") else { return };
        let e = json_to_js(&serde_json::json!({ "code": error.code.as_str(), "message": error.message }));
        if set(&m, "id", &JsValue::from(id)).is_some() && set(&m, "ok", &JsValue::FALSE).is_some() && set(&m, "error", &e).is_some() {
            self.post(&m, None);
        }
    }

    /// An event with JSON data.
    pub fn event(&self, name: &str, data: &Value) {
        self.event_js(name, &json_to_js(data), None);
    }

    /// An event whose data holds bytes under `data.data` (transferred).
    pub fn event_with_bytes(&self, name: &str, data: &Value, bytes: &[u8]) {
        let obj = json_to_js(data);
        let array = js_sys::Uint8Array::from(bytes);
        let buffer: JsValue = array.buffer().into();
        if js_sys::Reflect::set(&obj, &JsValue::from_str("data"), &array).is_ok() {
            self.event_js(name, &obj, Some(&buffer));
        }
    }

    fn event_js(&self, name: &str, data: &JsValue, transfer: Option<&JsValue>) {
        let Some(m) = self.envelope("evt") else { return };
        if set(&m, "name", &JsValue::from_str(name)).is_some() && set(&m, "data", data).is_some() {
            self.post(&m, transfer);
        }
    }

    /// Hears requests from the parent and queues them for the next frame.
    pub fn listen(&self, inbox: Inbox, ctx: egui::Context) {
        let me = self.clone();
        let handler = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
            if event.origin() != me.origin || !event.source().is_some_and(|s| js_sys::Object::is(&s, &me.parent)) {
                return;
            }
            let data = event.data();
            if !data.is_object() || get(&data, "ch").as_string().as_deref() != Some(me.channel.as_str()) {
                return;
            }
            if get(&data, "kind").as_string().as_deref() != Some("req") {
                return;
            }
            let Some(id) = get(&data, "id").as_f64().filter(|n| n.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(n)) else {
                web_sys::console::warn_1(&"Free PDF SDK: a request without a valid id was ignored".into());
                return;
            };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // checked: a whole number in u32's range
            let id = id as u32;
            if get(&data, "fps").as_f64() != Some(f64::from(PROTOCOL_VERSION)) {
                me.reply_err(id, &RpcError::new(ErrorCode::UnsupportedProtocol, format!("this viewer speaks protocol {PROTOCOL_VERSION}")));
                return;
            }
            match parse(&data) {
                Ok((method, params, bytes)) => {
                    if let Ok(mut q) = inbox.try_borrow_mut()
                        && q.len() < MAX_QUEUED
                    {
                        let deadline = get(&data, "deadline").as_f64().filter(|d| d.is_finite());
                        q.push_back(Incoming { id, method, params, data: bytes, deadline });
                    } else {
                        me.reply_err(id, &RpcError::new(ErrorCode::Busy, "the viewer is busy; try again"));
                        return;
                    }
                    ctx.request_repaint();
                }
                Err(e) => me.reply_err(id, &e),
            }
        });
        if self.window.add_event_listener_with_callback("message", handler.as_ref().unchecked_ref()).is_ok() {
            // The listener lives as long as the page (one viewer per iframe).
            handler.forget();
        }
    }
}

/// The method, the parameters without `data`, and `data`'s bytes.
fn parse(data: &JsValue) -> Result<(String, Value, Option<Vec<u8>>), RpcError> {
    let method = get(data, "method").as_string().ok_or_else(|| RpcError::bad_request("method must be a string"))?;
    let params = get(data, "params");
    if params.is_undefined() || params.is_null() {
        return Ok((method, Value::Null, None));
    }
    if !params.is_object() || js_sys::Array::is_array(&params) {
        return Err(RpcError::bad_request("params must be an object"));
    }
    let raw = get(&params, "data");
    let bytes = if raw.is_undefined() || raw.is_null() {
        None
    } else {
        let array = if let Some(a) = raw.dyn_ref::<js_sys::Uint8Array>() {
            a.clone()
        } else if raw.is_instance_of::<js_sys::ArrayBuffer>() {
            js_sys::Uint8Array::new(&raw)
        } else {
            return Err(RpcError::bad_request("params.data must be an ArrayBuffer or a Uint8Array"));
        };
        let len = usize::try_from(array.length()).unwrap_or(usize::MAX);
        if len > limits::MAX_PDF_BYTES {
            return Err(RpcError::new(ErrorCode::TooLarge, "params.data is too large"));
        }
        Some(array.to_vec())
    };
    // The rest goes through JSON: a copy of params without `data`.
    let copy = js_sys::Object::assign(&js_sys::Object::new(), params.unchecked_ref());
    let _ = js_sys::Reflect::delete_property(&copy, &JsValue::from_str("data"));
    let text = js_sys::JSON::stringify(&copy).map_err(|_| RpcError::bad_request("params must be plain JSON data"))?;
    let text = text.as_string().unwrap_or_default();
    if text.len() > limits::MAX_XFDF_BYTES.saturating_add(64 * 1024) {
        return Err(RpcError::new(ErrorCode::TooLarge, "params are too large"));
    }
    let value: Value = serde_json::from_str(&text).map_err(|e| RpcError::bad_request(format!("params: {e}")))?;
    Ok((method, value, bytes))
}

fn get(obj: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(obj, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn set(obj: &js_sys::Object, key: &str, value: &JsValue) -> Option<()> {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), value).ok().filter(|ok| *ok).map(|_| ())
}

/// A JSON value as a JS value (through JSON text: plain data only).
pub fn json_to_js(v: &Value) -> JsValue {
    serde_json::to_string(v).ok().and_then(|s| js_sys::JSON::parse(&s).ok()).unwrap_or(JsValue::NULL)
}

/// Posts an `error` event to the parent when the viewer could not start, without a Bridge
/// (best effort: the parent may not be listening yet).
pub fn report_start_failure(message: &str) {
    if let Ok(bridge) = Bridge::new() {
        bridge.event("error", &serde_json::json!({ "code": "INTERNAL", "message": message }));
    }
}

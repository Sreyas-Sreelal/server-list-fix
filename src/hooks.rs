use retour::static_detour;
use std::{
    error::Error,
    ffi::{c_char, c_int, CStr, CString},
    iter, mem,
    mem::size_of,
};
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Networking::WinSock::SOCKET,
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    },
};

// just a simple macro to convert Rust &str to C string
macro_rules! unsafe_cstr {
    ($e: expr) => {{
        union Transmute {
            src: &'static str,
            dst: &'static CStr,
        }

        const _TRANSMUTE_CHECK: [(); size_of::<&'static str>()]
            = [(); size_of::<&'static CStr>()];

        const RES: &'static CStr = unsafe {
            (Transmute { src: concat!($e, "\0") }).dst
        };

        RES
    }}
}
// define our detour function
// we only going to re route gethostbyname and send function from winsock library of windows
static_detour! {
  static GetHostByName: unsafe extern "system" fn(*const c_char) -> c_int;
  static Send: unsafe extern "system" fn(SOCKET,*const c_char,c_int,c_int) -> c_int;
}

type FnGetHostByName = unsafe extern "system" fn(*const c_char) -> c_int;
type FnSend = unsafe extern "system" fn(SOCKET, *const c_char, c_int, c_int) -> c_int;

fn gethostbyname_hook(name: *const c_char) -> c_int {
    let c_str: &CStr = unsafe { CStr::from_ptr(name) };
    // client is trying to use gethostbyname to resolve lists.sa-mp.com
    // we don't want that, instead we replace the hostname by the sam.markski.ar
    if c_str.to_str().unwrap() == "lists.sa-mp.com" {
        let hostname = unsafe_cstr!("sam.markski.ar");
        unsafe { GetHostByName.call(hostname.as_ptr()) }
    } else {
        unsafe { GetHostByName.call(name) }
    }
}

fn send_hook(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int) -> c_int {
    let c_str: &CStr = unsafe { CStr::from_ptr(buf) };
    // client is trying to use socket functin send to send an http request
    // if the end point contains /0.3.7/ (http://lists.sa-mp.com/0.3.7/...) that means that this an request to samp's official listing
    // you can find these requests using tools like wireshark
    if c_str.to_str().unwrap().starts_with("GET /0.3.7/") {
        // replace the request with our request to sam.markski.ar/api/GetMasterlist?version=0.3.7
        let request = unsafe_cstr!(
            "GET /api/GetMasterlist?version=0.3.7 HTTP/1.1
Content-Type: text/html
Host: sam.markski.ar
Accept: text/html, */*
User-Agent: Mozilla/3.0 (compatible; SA:MP v0.3.7)\r\n\r\n"
        );
        unsafe {
            Send.call(
                s,
                request.as_ptr(),
                request.to_str().unwrap().len().try_into().unwrap(),
                flags,
            )
        }
    } else {
        unsafe { Send.call(s, buf, len, flags) }
    }
}

pub unsafe fn init_hooks() -> Result<(), Box<dyn Error>> {
    // initialised our hooks by getting address of gethostbyname and send from ws2_32 libary

    let address = get_module_symbol_address("ws2_32.dll", "gethostbyname")
        .expect("could not find 'gethostbyname' address");

    let target: FnGetHostByName = mem::transmute(address);

    GetHostByName
        .initialize(target, gethostbyname_hook)?
        .enable()?;

    let address =
        get_module_symbol_address("ws2_32.dll", "send").expect("could not find 'send' address");
    let target: FnSend = mem::transmute(address);

    Send.initialize(target, send_hook)?.enable()?;

    Ok(())
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
    let module = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    let symbol = CString::new(symbol).unwrap();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr() as _)).unwrap();
        GetProcAddress(handle, PCSTR(symbol.as_ptr() as _)).map(|func| func as usize)
    }
}

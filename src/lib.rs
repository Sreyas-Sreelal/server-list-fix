#![feature(naked_functions)]

mod hooks;
mod proxy_dll;

use hooks::init_hooks;
use proxy_dll::initialise_library_functions;

use std::{error::Error, iter, os::raw::c_void, str::from_utf8};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, HANDLE, HMODULE, TRUE},
        System::{
            LibraryLoader::{GetModuleFileNameA, LoadLibraryW},
            SystemInformation::GetSystemDirectoryW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
};

unsafe fn main() -> Result<(), Box<dyn Error>> {
    // begin with setting up a proxy dll for actual version.dll
    // obtain system directory to find the actual version.dll
    // our version.dll will be loaded by the game as it's used by almost all windows applications
    let mut addr = [0; 100];
    GetSystemDirectoryW(Some(&mut addr));

    let mut addr: Vec<u16> = addr
        .to_vec()
        .iter()
        .filter_map(|x| if *x as u8 != b'\0' { Some(*x) } else { None })
        .collect();

    addr.append(
        &mut "\\version.dll"
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<u16>>(),
    );

    // load the actual version.dll into process
    let handle = LoadLibraryW(PCWSTR(addr.as_ptr()));
    
    initialise_library_functions(handle.unwrap())?;

    // get the name of executable that loaded our dll
    let mut name = [0; 255];
    GetModuleFileNameA(HMODULE(0), &mut name);
    let name: Vec<u8> = name
        .to_vec()
        .iter()
        .filter_map(|x| if *x != b'\0' { Some(*x) } else { None })
        .collect();
    let name = from_utf8(&name).unwrap();
    
    // we only need to initialise the hooks for samp.exe 
    // if anything else loaded our dll for example gta_sa.exe we're going to ignore
    if name.contains("samp.exe") {
        init_hooks()?;
    }

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        // AllocConsole().unwrap();
        BOOL::from(main().is_ok())
    } else {
        TRUE
    }
}

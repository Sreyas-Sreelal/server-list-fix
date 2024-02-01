use paste::paste;
use std::{arch::asm, error::Error, ffi::CString};
use windows::{
    core::PCSTR,
    Win32::{Foundation::HMODULE, System::LibraryLoader::GetProcAddress},
};

// basically create a proxy function to all exported functions found in version.dll
macro_rules! wrapper_gen_func {
    ($name:tt) => {
        paste! {
            #[allow(non_upper_case_globals)]
            static mut [<p $name>]: usize = 0;
            #[naked]
            #[no_mangle]
            pub extern "system" fn $name() {
                unsafe {
                    asm!(
                        "jmp  *({})",
                        sym [<p $name>] ,
                        options(noreturn, att_syntax),
                    );
                }
            }
        }
    };
}

// to view the list of exported functions use tools like dumpbin
// dumpbin /exports version.dll
wrapper_gen_func!(GetFileVersionInfoA);
wrapper_gen_func!(GetFileVersionInfoByHandle);
wrapper_gen_func!(GetFileVersionInfoExW);
wrapper_gen_func!(GetFileVersionInfoSizeA);
wrapper_gen_func!(GetFileVersionInfoSizeExW);
wrapper_gen_func!(GetFileVersionInfoSizeW);
wrapper_gen_func!(GetFileVersionInfoW);
wrapper_gen_func!(VerFindFileA);
wrapper_gen_func!(VerFindFileW);
wrapper_gen_func!(VerInstallFileA);
wrapper_gen_func!(VerInstallFileW);
wrapper_gen_func!(VerLanguageNameA);
wrapper_gen_func!(VerLanguageNameW);
wrapper_gen_func!(VerQueryValueA);
wrapper_gen_func!(VerQueryValueW);

pub unsafe fn initialise_library_functions(handle: HMODULE) -> Result<(), Box<dyn Error>> {
    // macro used to initialise our global variables with appropriate addresses of corresponding functions found in loaded dll
    macro_rules! init_address {
        ($name:tt) => {
            paste! {
                let c_str = CString::new(stringify!($name)).unwrap();
                [<p $name>] = GetProcAddress(handle, PCSTR(c_str.as_ptr() as _)).unwrap() as usize;
            }
        };
    }

    init_address!(GetFileVersionInfoA);
    init_address!(GetFileVersionInfoByHandle);
    init_address!(GetFileVersionInfoExW);
    init_address!(GetFileVersionInfoSizeA);
    init_address!(GetFileVersionInfoSizeExW);
    init_address!(GetFileVersionInfoSizeW);
    init_address!(GetFileVersionInfoW);
    init_address!(VerFindFileA);
    init_address!(VerFindFileW);
    init_address!(VerInstallFileA);
    init_address!(VerInstallFileW);
    init_address!(VerLanguageNameA);
    init_address!(VerLanguageNameW);
    init_address!(VerQueryValueA);
    init_address!(VerQueryValueW);

    Ok(())
}

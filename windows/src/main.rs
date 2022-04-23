use firn::arch::x86;
use firn::arch::x86::device::Cmos;
use firn::arch::x86::{Cpu, Feature};
use firn::cpu::Restrict;
use firn::mem::{BasicMem, Eeprom, MemMap};
use firn::System;
use std::{mem, ptr, thread};
use windows::core::PCSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, LoadCursorW, MessageBoxA,
    PostQuitMessage, RegisterClassExA, TranslateMessage, CW_USEDEFAULT, IDC_ARROW,
    MB_ICONEXCLAMATION, MB_OK, MSG, WM_DESTROY, WNDCLASSEXA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

fn main() -> windows::core::Result<()> {
    if unsafe { create_window()? }.is_none() {
        return Ok(());
    }

    let mut sys = create_sys();
    thread::spawn(move || {
        sys.run();
    });

    unsafe {
        let mut message = MSG::default();
        while GetMessageA(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }

    Ok(())
}

fn create_sys() -> System<Cpu> {
    let mem = BasicMem::new(640 * 1024);
    let eeprom = Eeprom::new_with_size(256 * 1024, x86::DEFAULT_BIOS);

    let mut map = MemMap::new(1024 * 1024);
    map.map_full(mem);
    map.map_from(0xc0000, 0xfffff, eeprom);

    let mut cpu = Cpu::new();
    cpu.add_feature(Feature::InstrCpu1);

    let cmos = Cmos::new_current_time();

    let mut sys = System::new(cpu, map);
    sys.add_device(cmos);

    sys
}

unsafe fn create_window() -> windows::core::Result<Option<HWND>> {
    let instance = GetModuleHandleA(None);
    let window_class = WNDCLASSEXA {
        cbSize: mem::size_of::<WNDCLASSEXA>() as u32,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        lpszClassName: PCSTR(b"Window\0".as_ptr()),
        ..Default::default()
    };

    let value = RegisterClassExA(&window_class);
    if value == 0 {
        MessageBoxA(
            None,
            "Failed to register window!",
            "Error",
            MB_ICONEXCLAMATION | MB_OK,
        );
        return Ok(None);
    }

    let window = CreateWindowExA(
        Default::default(),
        "Window",
        "Firn",
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        800,
        600,
        None,
        None,
        instance,
        ptr::null(),
    );

    if window.0 == 0 {
        MessageBoxA(
            None,
            "Failed to create window!",
            "Error",
            MB_ICONEXCLAMATION | MB_OK,
        );
        return Ok(None);
    }

    Ok(Some(window))
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => return DefWindowProcA(window, message, wparam, lparam),
    }

    LRESULT(0)
}

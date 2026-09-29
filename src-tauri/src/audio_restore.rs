/// On startup, restore the volume of every audio session that belongs to our
/// process tree to 1.0, fixing any leftover low-volume state from a previous run.

#[cfg(not(windows))]
pub fn restore_our_volume() {}

#[cfg(windows)]
pub fn restore_our_volume() {
    use std::collections::HashSet;
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{
        eMultimedia, eRender, IAudioSessionControl, IAudioSessionControl2,
        IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let root = std::process::id();

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        // Collect our process tree (main + WebView2 children)
        let mut our_pids: HashSet<u32> = HashSet::new();
        our_pids.insert(root);
        if let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    if entry.th32ParentProcessID == root {
                        our_pids.insert(entry.th32ProcessID);
                    }
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = windows::Win32::Foundation::CloseHandle(snap);
        }

        // Enumerate audio sessions and restore ours to 1.0
        let Ok(enum_dev): Result<IMMDeviceEnumerator, _> =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return;
        };
        let Ok(device) = enum_dev.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
            return;
        };
        let Ok(mgr): Result<IAudioSessionManager2, _> = device.Activate(CLSCTX_ALL, None) else {
            return;
        };
        let Ok(session_enum) = mgr.GetSessionEnumerator() else {
            return;
        };
        let Ok(count) = session_enum.GetCount() else {
            return;
        };

        for i in 0..count {
            let ctrl: IAudioSessionControl = match session_enum.GetSession(i) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let ctrl2: IAudioSessionControl2 = match ctrl.cast() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let pid = ctrl2.GetProcessId().unwrap_or(0);
            if !our_pids.contains(&pid) {
                continue;
            }
            let simple: ISimpleAudioVolume = match ctrl.cast() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let _ = simple.SetMasterVolume(1.0, std::ptr::null::<windows::core::GUID>());
        }
    }
}

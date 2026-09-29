#[cfg(windows)]
pub use imp::{duck, unduck};

#[cfg(not(windows))]
pub fn duck(_our_pid: u32) {}

#[cfg(not(windows))]
pub fn unduck(_our_pid: u32) {}

#[cfg(windows)]
mod imp {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
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

    static SAVED: OnceLock<Mutex<Vec<(u32, f32)>>> = OnceLock::new();

    fn saved() -> &'static Mutex<Vec<(u32, f32)>> {
        SAVED.get_or_init(|| Mutex::new(Vec::new()))
    }

    /// Returns our PID plus all direct child PIDs (e.g. WebView2 renderer processes).
    /// WebView2 runs audio under a child process, so we must skip those too.
    fn our_process_tree(root_pid: u32) -> HashSet<u32> {
        let mut pids = HashSet::new();
        pids.insert(root_pid);

        unsafe {
            let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) => h,
                Err(_) => return pids,
            };

            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    if entry.th32ParentProcessID == root_pid {
                        pids.insert(entry.th32ProcessID);
                    }
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = windows::Win32::Foundation::CloseHandle(snap);
        }

        pids
    }

    unsafe fn collect_other_sessions(
        skip_pids: &HashSet<u32>,
    ) -> Option<Vec<(u32, ISimpleAudioVolume)>> {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enum_dev: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enum_dev.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        let mgr: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None).ok()?;
        let session_enum = mgr.GetSessionEnumerator().ok()?;
        let count = session_enum.GetCount().ok()?;

        let mut out = Vec::new();
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
            // Skip system audio (pid 0) and our process tree (main + WebView2 children)
            if pid == 0 || skip_pids.contains(&pid) {
                continue;
            }
            let simple: ISimpleAudioVolume = match ctrl.cast() {
                Ok(v) => v,
                Err(_) => continue,
            };
            out.push((pid, simple));
        }
        Some(out)
    }

    pub fn duck(our_pid: u32) {
        let skip = our_process_tree(our_pid);
        unsafe {
            let sessions = match collect_other_sessions(&skip) {
                Some(s) => s,
                None => return,
            };
            let mut saved = saved().lock().unwrap();
            saved.clear();
            for (pid, simple) in &sessions {
                let vol = simple.GetMasterVolume().unwrap_or(1.0);
                saved.push((*pid, vol));
                let _ = simple.SetMasterVolume(0.15, std::ptr::null::<windows::core::GUID>());
            }
        }
    }

    pub fn unduck(our_pid: u32) {
        let saved_vols: Vec<(u32, f32)> = std::mem::take(&mut *saved().lock().unwrap());
        if saved_vols.is_empty() {
            return;
        }
        let skip = our_process_tree(our_pid);
        unsafe {
            let sessions = match collect_other_sessions(&skip) {
                Some(s) => s,
                None => return,
            };
            let mut saved_it = saved_vols.into_iter();
            for (_pid, simple) in &sessions {
                if let Some((_, vol)) = saved_it.next() {
                    let _ = simple.SetMasterVolume(
                        vol,
                        std::ptr::null::<windows::core::GUID>(),
                    );
                }
            }
        }
    }
}

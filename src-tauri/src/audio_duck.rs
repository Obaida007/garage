// Audio ducking: lowers the volume of all other applications while an announcement plays,
// then restores their original volumes when the announcement ends.

#[cfg(windows)]
pub use imp::{duck, unduck};

#[cfg(not(windows))]
pub fn duck(_our_pid: u32) {}

#[cfg(not(windows))]
pub fn unduck(_our_pid: u32) {}

#[cfg(windows)]
mod imp {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{
        eMultimedia, eRender, IAudioSessionControl, IAudioSessionControl2,
        IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    // Saved (pid, original_volume) in session-enumeration order, for non-our-pid sessions.
    static SAVED: Lazy<Mutex<Vec<(u32, f32)>>> = Lazy::new(|| Mutex::new(Vec::new()));

    unsafe fn collect_other_sessions(
        skip_pid: u32,
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
            // Skip system audio (pid 0) and our own process
            if pid == 0 || pid == skip_pid {
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
        unsafe {
            let sessions = match collect_other_sessions(our_pid) {
                Some(s) => s,
                None => return,
            };
            let mut saved = SAVED.lock().unwrap();
            saved.clear();
            for (pid, simple) in &sessions {
                let vol = simple.GetMasterVolume().unwrap_or(1.0);
                saved.push((*pid, vol));
                let _ = simple.SetMasterVolume(0.1, std::ptr::null::<windows::core::GUID>());
            }
        }
    }

    pub fn unduck(our_pid: u32) {
        let saved: Vec<(u32, f32)> = std::mem::take(&mut *SAVED.lock().unwrap());
        if saved.is_empty() {
            return;
        }
        unsafe {
            let sessions = match collect_other_sessions(our_pid) {
                Some(s) => s,
                None => return,
            };
            // Sessions are re-enumerated in the same order — match them in order.
            let mut saved_it = saved.into_iter();
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

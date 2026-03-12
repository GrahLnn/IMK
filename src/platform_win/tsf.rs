use windows::core::GUID;
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::TextServices::{
    CLSID_TF_InputProcessorProfiles, CLSID_TF_ThreadMgr, ITfCompartment, ITfCompartmentMgr,
    ITfInputProcessorProfiles, ITfThreadMgr, GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION,
    GUID_COMPARTMENT_KEYBOARD_OPENCLOSE, GUID_TFCAT_TIP_KEYBOARD, TF_CONVERSIONMODE_ALPHANUMERIC,
    TF_CONVERSIONMODE_NATIVE,
};

use crate::platform_win::ime::InputMode;

const LANG_ZH_CN: u16 = 0x0804;

pub struct TsfController;

impl TsfController {
    pub fn new() -> Self {
        Self
    }

    pub fn is_english(&self) -> Option<bool> {
        with_thread_mgr(|mgr, _client_id| {
            let cmgr = unsafe { mgr.GetGlobalCompartment().ok()? };
            let conversion =
                get_compartment(&cmgr, &GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION)?;
            let value = read_compartment_i32(&conversion)?;
            Some((value as u32 & TF_CONVERSIONMODE_NATIVE) == 0)
        })
    }

    pub fn set_mode(&self, mode: InputMode) -> bool {
        if let Some(result) = with_thread_mgr(|mgr, client_id| {
            let cmgr = unsafe { mgr.GetGlobalCompartment().ok()? };
            let open = get_compartment(&cmgr, &GUID_COMPARTMENT_KEYBOARD_OPENCLOSE)?;
            let conversion =
                get_compartment(&cmgr, &GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION)?;

            let _ = write_compartment_i32(&open, client_id, 1);
            let desired = match mode {
                InputMode::English => TF_CONVERSIONMODE_ALPHANUMERIC as i32,
                InputMode::Chinese => TF_CONVERSIONMODE_NATIVE as i32,
            };
            Some(write_compartment_i32(&conversion, client_id, desired))
        }) {
            if result {
                return true;
            }
        }

        if !try_activate_language_profile(LANG_ZH_CN) {
            return false;
        }

        with_thread_mgr(|mgr, client_id| {
            let cmgr = unsafe { mgr.GetGlobalCompartment().ok()? };
            let open = get_compartment(&cmgr, &GUID_COMPARTMENT_KEYBOARD_OPENCLOSE)?;
            let conversion =
                get_compartment(&cmgr, &GUID_COMPARTMENT_KEYBOARD_INPUTMODE_CONVERSION)?;
            let _ = write_compartment_i32(&open, client_id, 1);
            let desired = match mode {
                InputMode::English => TF_CONVERSIONMODE_ALPHANUMERIC as i32,
                InputMode::Chinese => TF_CONVERSIONMODE_NATIVE as i32,
            };
            Some(write_compartment_i32(&conversion, client_id, desired))
        })
        .unwrap_or(false)
    }
}

struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> Option<Self> {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_ok() {
                return Some(Self { initialized: true });
            }
            if hr == RPC_E_CHANGED_MODE {
                return Some(Self { initialized: false });
            }
            None
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

fn with_thread_mgr<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ITfThreadMgr, u32) -> Option<R>,
{
    let _com = ComGuard::new()?;
    let mgr: ITfThreadMgr =
        unsafe { CoCreateInstance(&CLSID_TF_ThreadMgr, None, CLSCTX_INPROC_SERVER).ok()? };
    let client_id = unsafe { mgr.Activate().ok()? };
    let result = f(&mgr, client_id);
    let _ = unsafe { mgr.Deactivate() };
    result
}

fn get_compartment(mgr: &ITfCompartmentMgr, guid: &GUID) -> Option<ITfCompartment> {
    unsafe { mgr.GetCompartment(guid).ok() }
}

fn read_compartment_i32(compartment: &ITfCompartment) -> Option<i32> {
    let value: VARIANT = unsafe { compartment.GetValue().ok()? };
    i32::try_from(&value).ok()
}

fn write_compartment_i32(compartment: &ITfCompartment, client_id: u32, value: i32) -> bool {
    let var = VARIANT::from(value);
    unsafe { compartment.SetValue(client_id, &var).is_ok() }
}

fn try_activate_language_profile(langid: u16) -> bool {
    let _com = match ComGuard::new() {
        Some(guard) => guard,
        None => return false,
    };

    let profiles: ITfInputProcessorProfiles = unsafe {
        match CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER) {
            Ok(profiles) => profiles,
            Err(_) => return false,
        }
    };

    let current_lang = unsafe { profiles.GetCurrentLanguage().ok() }.unwrap_or(langid);
    if current_lang != langid {
        return false;
    }

    let mut clsid = GUID::zeroed();
    let mut profile = GUID::zeroed();
    let result = unsafe {
        profiles
            .GetDefaultLanguageProfile(langid, &GUID_TFCAT_TIP_KEYBOARD, &mut clsid, &mut profile)
            .and_then(|_| profiles.ActivateLanguageProfile(&clsid, langid, &profile))
    };

    result.is_ok()
}

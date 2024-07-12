use std::{fs::OpenOptions, io::Write, sync::OnceLock};

use windows::{
    core::*,
    Win32::Networking::WinSock::{closesocket, recv, SEND_RECV_FLAGS, SOCKET},
};
use wsl_plugin_api_sys;

static API: OnceLock<wsl_plugin_api_sys::WSLPluginAPIV1> = OnceLock::new();

fn is_wsl_old(
    major: u32,
    minor: u32,
    revision: u32,
    api_version: &wsl_plugin_api_sys::WSLVersion,
) -> bool {
    api_version.Major < major
        || (api_version.Major == major && api_version.Minor < minor)
        || (api_version.Major == major
            && api_version.Minor == minor
            && api_version.Revision < revision)
}

fn write_log<S: AsRef<str>>(s: S) {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("C:\\wsl-plugin-demo.txt")
        .unwrap();
    f.write(s.as_ref().as_bytes()).unwrap();
}

fn read_from_socket(socket: SOCKET) -> String {
    let mut data = Vec::new();
    let mut buf = [0; 1024];
    loop {
        let result = unsafe { recv(socket, &mut buf, SEND_RECV_FLAGS(0)) };
        if result <= 0 {
            break;
        }
        data.extend_from_slice(&buf[..result.try_into().unwrap()]);
    }
    String::from_utf8_lossy(&data).to_string()
}

extern "C" fn on_vm_started(
    session: *const wsl_plugin_api_sys::WSLSessionInformation,
    settings: *const wsl_plugin_api_sys::WSLVmCreationSettings,
) -> wsl_plugin_api_sys::HRESULT {
    if session.is_null() || settings.is_null() {
        return 1;
    }
    let Some(api) = API.get() else {
        return 1;
    };
    let session = unsafe { session.as_ref().unwrap() };
    let settings = unsafe { settings.as_ref().unwrap() };
    write_log(format!(
        "VM created. SessionId = {} CustomConfigurationFlags = {}\n",
        session.SessionId, settings.CustomConfigurationFlags
    ));
    let execute_binary = api.ExecuteBinary.unwrap();
    let args: &mut [*const i8] = &mut [
        s!("/usr/bin/cat").as_ptr().cast(),
        s!("/proc/version").as_ptr().cast(),
        std::ptr::null(),
    ];
    let mut socket: wsl_plugin_api_sys::SOCKET = Default::default();
    let result = unsafe {
        execute_binary(
            session.SessionId,
            args[0],
            args.as_mut_ptr(),
            std::ptr::addr_of_mut!(socket),
        )
    };
    write_log("called execute_binary\n");
    if result != 0 {
        write_log(format!("Failed to create process: {}\n", result));
    } else {
        let socket = SOCKET(socket.try_into().unwrap());
        let data = read_from_socket(socket);
        write_log(format!("Kernel version info: {}", data));
        unsafe {
            closesocket(socket);
        }
    }

    0
}

extern "C" fn on_vm_stopping(
    session: *const wsl_plugin_api_sys::WSLSessionInformation,
) -> wsl_plugin_api_sys::HRESULT {
    let session = unsafe { session.as_ref().unwrap() };
    write_log(format!("VM stopping. SessionId = {}\n", session.SessionId));
    0
}

extern "C" fn on_distribution_started(
    session: *const wsl_plugin_api_sys::WSLSessionInformation,
    distribution: *const wsl_plugin_api_sys::WSLDistributionInformation,
) -> wsl_plugin_api_sys::HRESULT {
    let session = unsafe { session.as_ref().unwrap() };
    let distribution = unsafe { distribution.as_ref().unwrap() };
    let name = unsafe { PCWSTR::from_raw(distribution.Name).to_string().unwrap() };
    let package_name = if distribution.PackageFamilyName.is_null() {
        "(null)".to_string()
    } else {
        unsafe {
            PCWSTR::from_raw(distribution.PackageFamilyName)
                .to_string()
                .unwrap()
        }
    };
    write_log(format!(
        "Distibution started. SessionId={}, name={}, package={}, PidNs={}, InitPid={}\n",
        session.SessionId, name, package_name, distribution.PidNamespace, distribution.InitPid
    ));
    0
}

extern "C" fn on_distribution_stopping(
    session: *const wsl_plugin_api_sys::WSLSessionInformation,
    distribution: *const wsl_plugin_api_sys::WSLDistributionInformation,
) -> wsl_plugin_api_sys::HRESULT {
    let session = unsafe { session.as_ref().unwrap() };
    let distribution = unsafe { distribution.as_ref().unwrap() };
    let name = unsafe { PCWSTR::from_raw(distribution.Name).to_string().unwrap() };
    let package_name = if distribution.PackageFamilyName.is_null() {
        "(null)".to_string()
    } else {
        unsafe {
            PCWSTR::from_raw(distribution.PackageFamilyName)
                .to_string()
                .unwrap()
        }
    };
    write_log(format!(
        "Distibution stopping. SessionId={}, name={}, package={}, PidNs={}, InitPid={}\n",
        session.SessionId, name, package_name, distribution.PidNamespace, distribution.InitPid
    ));
    0
}

#[no_mangle]
pub extern "C" fn WSLPluginAPIV1_EntryPoint(
    api: *const wsl_plugin_api_sys::WSLPluginAPIV1,
    hooks: *mut wsl_plugin_api_sys::WSLPluginHooksV1,
) -> wsl_plugin_api_sys::HRESULT {
    if api.is_null() || hooks.is_null() {
        return 1;
    }
    let api = unsafe { *api };

    write_log(format!(
        "Plugin loaded. WSL version: {}.{}.{}\n",
        api.Version.Major, api.Version.Minor, api.Version.Revision
    ));
    if is_wsl_old(2, 1, 2, &api.Version) {
        return 1;
    }

    let result = API.set(api);
    if result.is_err() {
        return 1;
    }
    let hooks = unsafe { hooks.as_mut().unwrap() };
    hooks.OnVMStarted = Some(on_vm_started);
    hooks.OnVMStopping = Some(on_vm_stopping);
    hooks.OnDistributionStarted = Some(on_distribution_started);
    hooks.OnDistributionStopping = Some(on_distribution_stopping);
    0
}

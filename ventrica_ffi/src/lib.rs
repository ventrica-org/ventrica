#![allow(unsafe_code, clippy::not_unsafe_ptr_arg_deref, unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use ventricad::Package;
use ventricad::{DEFAULT_SOCKET, DaemonClient, Message, Repo, Request};

#[repr(C)]
pub struct VentPackage {
    pub id: i64,
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub category: *const c_char,
    pub installed_at: i64,
    pub icon: *const c_char,
    pub native_description: *const c_char,
    pub run_dep_names: *const *const c_char,
    pub run_dep_names_count: usize,
}

impl Drop for VentPackage {
    fn drop(&mut self) {
        unsafe {
            free_cstr(self.name);
            free_cstr(self.version);
            free_cstr(self.description);
            free_cstr(self.category);
            free_cstr_nullable(self.icon);
            free_cstr_nullable(self.native_description);
            if !self.run_dep_names.is_null() && self.run_dep_names_count > 0 {
                let slice =
                    std::slice::from_raw_parts(self.run_dep_names, self.run_dep_names_count);
                for &ptr in slice {
                    free_cstr_nullable(ptr);
                }
                drop(Vec::from_raw_parts(
                    self.run_dep_names as *mut *const c_char,
                    self.run_dep_names_count,
                    self.run_dep_names_count,
                ));
            }
        }
    }
}

#[repr(C)]
pub struct VentGeneration {
    pub number: u32,
    pub created_at: i64,
    pub description: *const c_char,
}

impl Drop for VentGeneration {
    fn drop(&mut self) {
        unsafe { free_cstr_nullable(self.description) }
    }
}

#[repr(C)]
pub struct VentRepo {
    pub id: i64,
    pub name: *const c_char,
    pub url: *const c_char,
    pub icon: *const c_char,
    pub added_at: i64,
}

impl Drop for VentRepo {
    fn drop(&mut self) {
        unsafe {
            free_cstr(self.name);
            free_cstr(self.url);
        }
    }
}

#[repr(C)]
pub struct VentError {
    pub message: *const c_char,
}

fn package_to_vent_package(pkg: Package) -> *mut VentPackage {
    let run_dep_names: Vec<String> = pkg
        .dependencies
        .as_ref()
        .map(|deps| {
            deps.iter()
                .filter(|d| !d.is_build.unwrap_or(false))
                .filter_map(|d| d.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let (dep_names_ptr, dep_names_count) =
        make_cstr_array(run_dep_names.iter().map(String::as_str));

    Box::into_raw(Box::new(VentPackage {
        id: pkg.id.unwrap_or_default(),
        name: cs(pkg.name),
        version: cs(pkg.version),
        description: cs(pkg.description),
        category: cs(pkg.category.unwrap_or_default()),
        installed_at: pkg.installed_at.unwrap_or_default(),
        icon: cs_opt(pkg.icon),
        native_description: cs_opt(pkg.native_depiction),
        run_dep_names: dep_names_ptr,
        run_dep_names_count: dep_names_count,
    }))
}

fn make_cstr_array<'a>(iter: impl Iterator<Item = &'a str>) -> (*const *const c_char, usize) {
    let mut v: Vec<*const c_char> = iter.map(cs).collect();
    v.shrink_to_fit();
    let count = v.len();
    if count == 0 {
        (std::ptr::null(), 0)
    } else {
        let ptr = v.as_ptr();
        std::mem::forget(v);
        (ptr, count)
    }
}

fn cs(s: impl Into<Vec<u8>>) -> *const c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("<invalid>").unwrap())
        .into_raw()
}

fn cs_opt(s: Option<String>) -> *const c_char {
    s.map(|v| cs(v)).unwrap_or(std::ptr::null())
}

unsafe fn free_cstr(p: *const c_char) {
    if !p.is_null() {
        drop(CString::from_raw(p as *mut c_char));
    }
}

unsafe fn free_cstr_nullable(p: *const c_char) {
    if !p.is_null() {
        drop(CString::from_raw(p as *mut c_char));
    }
}

unsafe fn set_error(out: *mut *mut VentError, msg: impl std::fmt::Display) {
    if out.is_null() {
        return;
    }
    let msg = CString::new(msg.to_string())
        .unwrap_or_else(|_| CString::new("error message contained a null byte").unwrap());
    *out = Box::into_raw(Box::new(VentError {
        message: msg.into_raw(),
    }));
}

unsafe fn clear_error(out: *mut *mut VentError) {
    if !out.is_null() {
        *out = std::ptr::null_mut();
    }
}

unsafe fn cstr_to_str<'a>(s: *const c_char, field: &str) -> Result<&'a str, String> {
    if s.is_null() {
        return Err(format!("{field} must not be null"));
    }
    CStr::from_ptr(s)
        .to_str()
        .map_err(|_| format!("{field} is not valid UTF-8"))
}

fn daemon_call(req: &Request) -> Result<Option<serde_json::Value>, String> {
    let mut client = DaemonClient::connect_to(DEFAULT_SOCKET)
        .map_err(|e| format!("cannot connect to ventricad: {e}"))?;

    let mut data: Option<serde_json::Value> = None;
    let mut err: Option<String> = None;

    client
        .send(req, |msg| match msg {
            Message::Error(e) => err = Some(e),
            Message::Data(v) => data = Some(v),
            _ => {}
        })
        .map_err(|e| format!("socket I/O error: {e}"))?;

    if let Some(e) = err { Err(e) } else { Ok(data) }
}

fn into_ptr_array<T>(mut v: Vec<*mut T>) -> (*mut *mut T, usize) {
    v.shrink_to_fit();
    let len = v.len();
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    (ptr, len)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_install(
    names: *const *const c_char,
    names_count: usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let mut pkg_names: Vec<String> = Vec::new();
    if !names.is_null() && names_count > 0 {
        let slice = std::slice::from_raw_parts(names, names_count);
        for &ptr in slice {
            match cstr_to_str(ptr, "name") {
                Ok(n) => pkg_names.push(n.to_owned()),
                Err(e) => {
                    set_error(out_err, e);
                    return -1;
                }
            }
        }
    }
    match daemon_call(&Request::Install { names: pkg_names }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_remove(
    names: *const *const c_char,
    names_count: usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let mut pkg_names: Vec<String> = Vec::new();
    if !names.is_null() {
        let slice = std::slice::from_raw_parts(names, names_count);
        for &ptr in slice {
            match cstr_to_str(ptr, "name") {
                Ok(n) => pkg_names.push(n.to_owned()),
                Err(e) => {
                    set_error(out_err, e);
                    return -1;
                }
            }
        }
    } else {
        set_error(out_err, "name must not be null");
        return -1;
    }

    match daemon_call(&Request::Remove { names: pkg_names }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_upgrade(
    names: *const *const c_char,
    names_count: usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let mut pkg_names: Vec<String> = Vec::new();
    if !names.is_null() && names_count > 0 {
        let slice = std::slice::from_raw_parts(names, names_count);
        for &ptr in slice {
            match cstr_to_str(ptr, "name") {
                Ok(n) => pkg_names.push(n.to_owned()),
                Err(e) => {
                    set_error(out_err, e);
                    return -1;
                }
            }
        }
    }
    match daemon_call(&Request::Upgrade { names: pkg_names }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_rollback(generation: u32, out_err: *mut *mut VentError) -> c_int {
    clear_error(out_err);
    match daemon_call(&Request::Rollback {
        generation: Some(generation),
    }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_packages(
    arr_out: *mut *mut *mut VentPackage,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    match daemon_call(&Request::ListPackages) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<Package> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let items: Vec<*mut VentPackage> =
                rows.into_iter().map(package_to_vent_package).collect();
            let (ptr, len) = into_ptr_array(items);
            *arr_out = ptr;
            *count_out = len;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_generations(
    arr_out: *mut *mut *mut VentGeneration,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    match daemon_call(&Request::ListGenerations) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<serde_json::Value> =
                data.and_then(|v| v.as_array().cloned()).unwrap_or_default();
            let items: Vec<*mut VentGeneration> = rows
                .iter()
                .map(|r| {
                    Box::into_raw(Box::new(VentGeneration {
                        number: r["number"].as_u64().unwrap_or(0) as u32,
                        created_at: r["created_at"].as_i64().unwrap_or(0),
                        description: cs_opt(r["description"].as_str().map(str::to_owned)),
                    }))
                })
                .collect();
            let (ptr, len) = into_ptr_array(items);
            *arr_out = ptr;
            *count_out = len;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_current_generation(out_err: *mut *mut VentError) -> i32 {
    clear_error(out_err);
    match daemon_call(&Request::ListGenerations) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => data
            .and_then(|v| {
                v.as_array()?
                    .iter()
                    .find(|g| g["current"].as_bool() == Some(true))
                    .and_then(|g| g["number"].as_u64())
                    .map(|n| n as i32)
            })
            .unwrap_or(0),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_add_repo(
    url: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&Request::AddRepo { url: url_str }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_remove_repo(
    url: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&Request::RemoveRepo { url: url_str }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_repos(
    arr_out: *mut *mut *mut VentRepo,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    match daemon_call(&Request::ListRepos) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<Repo> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let items: Vec<*mut VentRepo> = rows
                .into_iter()
                .map(|r| {
                    Box::into_raw(Box::new(VentRepo {
                        id: r.id.unwrap_or_default(),
                        name: cs(r.name),
                        url: cs_opt(r.url),
                        icon: cs_opt(r.icon),
                        added_at: r.installed_at.unwrap_or_default(),
                    }))
                })
                .collect();
            let (ptr, len) = into_ptr_array(items);
            *arr_out = ptr;
            *count_out = len;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_search(
    query: *const c_char,
    arr_out: *mut *mut *mut VentPackage,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let q = match cstr_to_str(query, "query") {
        Ok(q) => q.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&Request::Search { query: q }) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<Package> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let items: Vec<*mut VentPackage> =
                rows.into_iter().map(package_to_vent_package).collect();
            let (ptr, len) = into_ptr_array(items);
            *arr_out = ptr;
            *count_out = len;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_repo_packages(
    url: *const c_char,
    arr_out: *mut *mut *mut VentPackage,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&Request::ListRepoPackages { url: url_str }) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<Package> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let items: Vec<*mut VentPackage> =
                rows.into_iter().map(package_to_vent_package).collect();
            let (ptr, len) = into_ptr_array(items);
            *arr_out = ptr;
            *count_out = len;
            0
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_pkg_free(pkg: *mut VentPackage) {
    if !pkg.is_null() {
        drop(Box::from_raw(pkg));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_pkg_array_free(arr: *mut *mut VentPackage, count: usize) {
    free_array(arr, count);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_gen_free(generation: *mut VentGeneration) {
    if !generation.is_null() {
        drop(Box::from_raw(generation));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_gen_array_free(arr: *mut *mut VentGeneration, count: usize) {
    free_array(arr, count);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_free(repo: *mut VentRepo) {
    if !repo.is_null() {
        drop(Box::from_raw(repo));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_array_free(arr: *mut *mut VentRepo, count: usize) {
    free_array(arr, count);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_search_result_free(result: *mut VentPackage) {
    ventrica_pkg_free(result);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_search_result_array_free(
    arr: *mut *mut VentPackage,
    count: usize,
) {
    ventrica_pkg_array_free(arr, count);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_package_free(pkg: *mut VentPackage) {
    ventrica_pkg_free(pkg);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_package_array_free(
    arr: *mut *mut VentPackage,
    count: usize,
) {
    ventrica_pkg_array_free(arr, count);
}

unsafe fn free_array<T>(arr: *mut *mut T, count: usize) {
    if arr.is_null() {
        return;
    }
    let slice = std::slice::from_raw_parts_mut(arr, count);
    for ptr in slice.iter() {
        if !ptr.is_null() {
            drop(Box::from_raw(*ptr));
        }
    }
    drop(Vec::from_raw_parts(arr, count, count));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_error_message(err: *const VentError) -> *const c_char {
    (*err).message
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_error_free(err: *mut VentError) {
    if !err.is_null() {
        drop(Box::from_raw(err));
    }
}

#![allow(unsafe_code, clippy::not_unsafe_ptr_arg_deref, unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use ventricad::{DaemonClient, Message, Request};
use ventricad::{PackageEntry, RepoRecord};

pub struct VentStore {
    socket_path: String,
}
#[repr(C)]
pub struct VentPackage {
    pub id: i64,
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub category: *const c_char,
    pub store_name: *const c_char,
    pub store_path: *const c_char,
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
            free_cstr(self.store_name);
            free_cstr(self.store_path);
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
pub struct VentSearchResult {
    pub repo_url: *const c_char,
    pub repo_name: *const c_char,
    pub name: *const c_char,
    pub version: *const c_char,
    pub store_name: *const c_char,
    pub filename: *const c_char,
    pub var_hash: *const c_char,
}

impl Drop for VentSearchResult {
    fn drop(&mut self) {
        unsafe {
            free_cstr(self.repo_url);
            free_cstr(self.repo_name);
            free_cstr(self.name);
            free_cstr(self.version);
            free_cstr(self.store_name);
            free_cstr(self.filename);
            free_cstr(self.var_hash);
        }
    }
}

#[repr(C)]
pub struct VentRepoPackage {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub category: *const c_char,
    pub icon: *const c_char,
    pub native_description: *const c_char,
    pub store_name: *const c_char,
    pub filename: *const c_char,
    pub var_hash: *const c_char,
    pub run_deps: *const *const c_char,
    pub run_deps_count: usize,
}

impl Drop for VentRepoPackage {
    fn drop(&mut self) {
        unsafe {
            free_cstr(self.name);
            free_cstr(self.version);
            free_cstr(self.description);
            free_cstr(self.category);
            free_cstr(self.store_name);
            free_cstr(self.filename);
            free_cstr(self.var_hash);
            free_cstr_nullable(self.icon);
            free_cstr_nullable(self.native_description);
            if !self.run_deps.is_null() && self.run_deps_count > 0 {
                let slice = std::slice::from_raw_parts(self.run_deps, self.run_deps_count);
                for &ptr in slice {
                    free_cstr_nullable(ptr);
                }
                drop(Vec::from_raw_parts(
                    self.run_deps as *mut *const c_char,
                    self.run_deps_count,
                    self.run_deps_count,
                ));
            }
        }
    }
}

pub struct VentError {
    message: CString,
}

/// Allocate a C string pointer array from an iterator of `&str`, returning
/// the raw pointer and the element count. The caller is responsible for
/// freeing every element and the array itself.
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
    *out = Box::into_raw(Box::new(VentError { message: msg }));
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

fn daemon_call(socket_path: &str, req: &Request) -> Result<Option<serde_json::Value>, String> {
    let mut client = DaemonClient::connect_to(socket_path)
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
pub unsafe extern "C" fn ventrica_store_open(
    socket_path: *const c_char,
    _user_path: *const c_char,
    out_err: *mut *mut VentError,
) -> *mut VentStore {
    clear_error(out_err);
    let path = if socket_path.is_null() {
        std::env::var(ventricad::SOCKET_ENV)
            .unwrap_or_else(|_| ventricad::DEFAULT_SOCKET.to_owned())
    } else {
        match cstr_to_str(socket_path, "socket_path") {
            Ok(s) => s.to_owned(),
            Err(e) => {
                set_error(out_err, e);
                return std::ptr::null_mut();
            }
        }
    };
    Box::into_raw(Box::new(VentStore { socket_path: path }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_store_open_default(
    out_err: *mut *mut VentError,
) -> *mut VentStore {
    ventrica_store_open(std::ptr::null(), std::ptr::null(), out_err)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_store_close(store: *mut VentStore) {
    if !store.is_null() {
        drop(Box::from_raw(store));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_install(
    store: *mut VentStore,
    recipe_path: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let recipe = match cstr_to_str(recipe_path, "recipe_path") {
        Ok(r) => r.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(
        &s.socket_path,
        &Request::Install {
            recipes: vec![recipe],
        },
    ) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_install_name(
    store: *mut VentStore,
    name: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    ventrica_install(store, name, out_err)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_remove(
    store: *mut VentStore,
    name: *const c_char,
    version: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let name_str = match cstr_to_str(name, "name") {
        Ok(n) => n.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    let ver = if version.is_null() {
        None
    } else {
        match cstr_to_str(version, "version") {
            Ok(v) => Some(v.to_owned()),
            Err(e) => {
                set_error(out_err, e);
                return -1;
            }
        }
    };
    match daemon_call(
        &s.socket_path,
        &Request::Remove {
            name: name_str,
            version: ver,
        },
    ) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_upgrade(
    store: *mut VentStore,
    names: *const *const c_char,
    names_count: usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
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
    match daemon_call(&s.socket_path, &Request::Upgrade { names: pkg_names }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_rollback(
    store: *mut VentStore,
    generation: u32,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    match daemon_call(
        &s.socket_path,
        &Request::Rollback {
            generation: Some(generation),
        },
    ) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_packages(
    store: *mut VentStore,
    arr_out: *mut *mut *mut VentPackage,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    match daemon_call(&s.socket_path, &Request::ListPackages) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<serde_json::Value> =
                data.and_then(|v| v.as_array().cloned()).unwrap_or_default();
            // Build a store_path → name map so dep names can be resolved
            // without parsing the path string (store_name format is "<name>-<version>").
            let sp_to_name: std::collections::HashMap<&str, &str> = rows
                .iter()
                .filter_map(|r| {
                    let sp = r["store_path"].as_str()?;
                    let name = r["name"].as_str()?;
                    Some((sp, name))
                })
                .collect();
            let items: Vec<*mut VentPackage> = rows
                .iter()
                .map(|r| {
                    let dep_names: Vec<&str> = r["run_dep_store_paths"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str())
                                .filter_map(|sp| sp_to_name.get(sp).copied())
                                .collect()
                        })
                        .unwrap_or_default();
                    let (dep_names_ptr, dep_names_count) = make_cstr_array(dep_names.into_iter());
                    Box::into_raw(Box::new(VentPackage {
                        id: r["id"].as_i64().unwrap_or(0),
                        name: cs(r["name"].as_str().unwrap_or("")),
                        version: cs(r["version"].as_str().unwrap_or("")),
                        description: cs(r["description"].as_str().unwrap_or("")),
                        category: cs(r["category"].as_str().unwrap_or("")),
                        store_name: cs(r["store_name"].as_str().unwrap_or("")),
                        store_path: cs(r["store_path"].as_str().unwrap_or("")),
                        installed_at: r["installed_at"].as_i64().unwrap_or(0),
                        icon: cs_opt(r["icon"].as_str().map(str::to_owned)),
                        native_description: cs_opt(
                            r["native_description"].as_str().map(str::to_owned),
                        ),
                        run_dep_names: dep_names_ptr,
                        run_dep_names_count: dep_names_count,
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
pub unsafe extern "C" fn ventrica_list_generations(
    store: *mut VentStore,
    arr_out: *mut *mut *mut VentGeneration,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    match daemon_call(&s.socket_path, &Request::ListGenerations) {
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
pub unsafe extern "C" fn ventrica_current_generation(
    store: *mut VentStore,
    out_err: *mut *mut VentError,
) -> i32 {
    clear_error(out_err);
    let s = &*store;
    match daemon_call(&s.socket_path, &Request::ListGenerations) {
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
    store: *mut VentStore,
    url: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&s.socket_path, &Request::AddRepo { url: url_str }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_remove_repo(
    store: *mut VentStore,
    url: *const c_char,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&s.socket_path, &Request::RemoveRepo { url: url_str }) {
        Ok(_) => 0,
        Err(e) => {
            set_error(out_err, e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_list_repos(
    store: *mut VentStore,
    arr_out: *mut *mut *mut VentRepo,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    match daemon_call(&s.socket_path, &Request::ListRepos) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<RepoRecord> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let items: Vec<*mut VentRepo> = rows
                .into_iter()
                .map(|r| {
                    Box::into_raw(Box::new(VentRepo {
                        id: r.id,
                        name: cs(r.name),
                        url: cs(r.url),
                        added_at: r.added_at,
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
    store: *mut VentStore,
    query: *const c_char,
    arr_out: *mut *mut *mut VentSearchResult,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let q = match cstr_to_str(query, "query") {
        Ok(q) => q.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&s.socket_path, &Request::Search { query: q }) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<serde_json::Value> =
                data.and_then(|v| v.as_array().cloned()).unwrap_or_default();
            let items: Vec<*mut VentSearchResult> = rows
                .iter()
                .map(|r| {
                    Box::into_raw(Box::new(VentSearchResult {
                        repo_url: cs(""),
                        repo_name: cs(r["repo"].as_str().unwrap_or("")),
                        name: cs(r["name"].as_str().unwrap_or("")),
                        version: cs(r["version"].as_str().unwrap_or("")),
                        store_name: cs(""),
                        filename: cs(""),
                        var_hash: cs(""),
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
pub unsafe extern "C" fn ventrica_list_repo_packages(
    store: *mut VentStore,
    url: *const c_char,
    arr_out: *mut *mut *mut VentRepoPackage,
    count_out: *mut usize,
    out_err: *mut *mut VentError,
) -> c_int {
    clear_error(out_err);
    let s = &*store;
    let url_str = match cstr_to_str(url, "url") {
        Ok(u) => u.to_owned(),
        Err(e) => {
            set_error(out_err, e);
            return -1;
        }
    };
    match daemon_call(&s.socket_path, &Request::ListRepoPackages { url: url_str }) {
        Err(e) => {
            set_error(out_err, e);
            -1
        }
        Ok(data) => {
            let rows: Vec<PackageEntry> = data
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let items: Vec<*mut VentRepoPackage> = rows
                .into_iter()
                .map(|r| {
                    let mut deps: Vec<*const c_char> =
                        r.run_deps.into_iter().map(|s| cs(s)).collect();
                    deps.shrink_to_fit();
                    let count = deps.len();
                    let run_deps = if count == 0 {
                        std::mem::forget(deps);
                        std::ptr::null()
                    } else {
                        let ptr = deps.as_ptr();
                        std::mem::forget(deps);
                        ptr
                    };
                    Box::into_raw(Box::new(VentRepoPackage {
                        name: cs(r.name),
                        version: cs(r.version),
                        description: cs(r.description),
                        category: cs(r.category),
                        icon: cs_opt(r.icon),
                        native_description: std::ptr::null(),
                        store_name: cs(r.store_name),
                        filename: cs(r.filename),
                        var_hash: cs(r.var_hash),
                        run_deps,
                        run_deps_count: count,
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
pub unsafe extern "C" fn ventrica_search_result_free(result: *mut VentSearchResult) {
    if !result.is_null() {
        drop(Box::from_raw(result));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_search_result_array_free(
    arr: *mut *mut VentSearchResult,
    count: usize,
) {
    free_array(arr, count);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_package_free(pkg: *mut VentRepoPackage) {
    if !pkg.is_null() {
        drop(Box::from_raw(pkg));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_repo_package_array_free(
    arr: *mut *mut VentRepoPackage,
    count: usize,
) {
    free_array(arr, count);
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
    (*err).message.as_ptr()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ventrica_error_free(err: *mut VentError) {
    if !err.is_null() {
        drop(Box::from_raw(err));
    }
}

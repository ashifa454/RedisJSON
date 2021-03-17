use redis_module::{raw as rawmod, RedisError, RedisString};
use redis_module::{Context, NotifyEvent, Status};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_void},
};

use crate::{
    redisjson::{Format, Path, RedisJSON},
    REDIS_JSON_TYPE,
};

use crate::Error;
use redis_module::key::RedisKeyWritable;
use serde_json::Value;
use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr::{null, null_mut};

//
// structs
//

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RedisModuleCtx {
    _unused: [u8; 0],
}

#[repr(C)]
pub enum JSONType {
    String = 0,
    Int = 1,
    Float = 2,
    Bool = 3,
    Object = 4,
    Array = 5,
    Null = 6,
    Err = 7,
}

//---------------------------------------------------------------------------------------------

struct JSONApiKey<'a> {
    key: RedisKeyWritable,
    redis_json: &'a mut RedisJSON,
}

type JSONApiKeyRef = *mut c_void;

impl<'a> JSONApiKey<'a> {
    pub fn new(
        ctx: *mut rawmod::RedisModuleCtx,
        key_str: *mut rawmod::RedisModuleString,
    ) -> Result<JSONApiKey<'a>, RedisError> {
        let ctx = Context::new(ctx);
        let key = ctx.open_with_redis_string(key_str);
        let res = key.get_value::<RedisJSON>(&REDIS_JSON_TYPE)?;

        if let Some(value) = res {
            Ok(JSONApiKey {
                key,
                redis_json: value,
            })
        } else {
            Err(RedisError::Str("Not a JSON key"))
        }
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_openKey(
    ctx: *mut rawmod::RedisModuleCtx,
    key_str: *mut rawmod::RedisModuleString,
) -> JSONApiKeyRef {
    match JSONApiKey::new(ctx, key_str) {
        Ok(key) => Box::into_raw(Box::new(key)) as JSONApiKeyRef,
        Err(e) => null_mut() as JSONApiKeyRef,
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_closeKey(json: JSONApiKeyRef) {
    if !json.is_null() {
        unsafe {
            Box::from_raw(json);
        }
    }
}

//---------------------------------------------------------------------------------------------

struct JSONApiPath<'a> {
    json_key: &'a JSONApiKey,
    path: Value,
}

type JSONApiPathRef = *mut c_void;

#[no_mangle]
pub extern "C" fn JSONAPI_getPath(
    json_key: JSONApiKeyRef,
    path: *const c_char,
) -> JSONApiPathRef {
    let ctx = Context::new(module_ctx);
    let key = ctx.open_with_redis_string(key_str);
    if let Ok(res) = key.get_value::<RedisJSON>(&REDIS_JSON_TYPE) {
        if let Some(value) = res {
            let p = unsafe { CStr::from_ptr(path).to_str().unwrap() };
            if let Ok(value) = value.get_first(p) {
                Box::into_raw(Box::new(value)) as *mut c_void
            } else {
                null_mut()
            }
        } else {
            null_mut()
        }
    } else {
        null_mut()
    }
}

#[no_mangle]
pub extern "C" fn JSONAPI_getInfo(
    redisjson: JSONApiKeyRef,
    _name: *mut c_void,
    jtype: *mut c_int,
    size: *mut libc::size_t,
) -> c_int {
    let t: c_int;
    if !redisjson.is_null() {
        let json = unsafe { &*(redisjson as *mut RedisJSON) };
        t = json.get_type_as_numeric();
    } else {
        t = JSONType::Err as c_int
    }
    unsafe {
        *jtype = t;
    }
    0
}

//---------------------------------------------------------------------------------------------

static REDISJSON_GETAPI: &str = concat!("RedisJSON_V1", "\0");

pub fn export_shared_api(ctx: &Context) {
    ctx.export_shared_api(
        &JSONAPI as *const RedisJSONAPI_V1 as *const c_void,
        REDISJSON_GETAPI.as_ptr() as *mut i8,
    );
}

static JSONAPI: RedisJSONAPI_V1 = RedisJSONAPI_V1 {
    openKey: JSONAPI_openKey,
    getPath: JSONAPI_getPath,
    getInfo: JSONAPI_getInfo,
    closeKey: JSONAPI_closeKey
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RedisJSONAPI_V1 {
    pub openKey: extern "C" fn(
        module_ctx: *mut rawmod::RedisModuleCtx,
        key_str: *mut rawmod::RedisModuleString,
        path: *const c_char,
    ) -> JSONApiKeyRef,
    pub getPath: extern "C" fn(
        json_key: JSONApiKeyRef,
        path: *const c_char,
    ) -> JSONApiPathRef,
    pub getInfo: extern "C" fn(json: JSONApiPathRef, name: *mut c_void, jtype: *mut c_int, size: *mut libc::size_t) -> c_int,
    pub closeKey: extern "C" fn(key: JSONApiKeyRef),
}

pub fn notify_keyspace_event(
    ctx: &Context,
    event_type: NotifyEvent,
    event: &str,
    keyname: &str,
) -> Status {
    ctx.notify_keyspace_event(event_type, event, keyname)
}

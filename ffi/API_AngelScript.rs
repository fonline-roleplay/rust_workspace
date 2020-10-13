dynamic_ffi!(AngelScriptApi, 
    pub fn Script_String(str_: *const ::std::os::raw::c_char) -> *mut ScriptString;
);

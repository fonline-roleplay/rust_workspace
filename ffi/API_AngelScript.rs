dynamic_ffi!(AngelScriptApi, 
/*
    pub fn Script_RegisterObjectType(
        obj: *const ::std::os::raw::c_char,
        byteSize: ::std::os::raw::c_int,
        flags: asDWORD,
    ) -> ::std::os::raw::c_int;

    pub fn Script_RegisterObjectProperty(
        obj: *const ::std::os::raw::c_char,
        declaration: *const ::std::os::raw::c_char,
        byteOffset: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    pub fn Script_RegisterObjectMethod(
        obj: *const ::std::os::raw::c_char,
        declaration: *const ::std::os::raw::c_char,
        funcPointer: *const asSFuncPtr,
        callConv: asDWORD,
    ) -> ::std::os::raw::c_int;

    pub fn Script_RegisterObjectBehaviour(
        obj: *const ::std::os::raw::c_char,
        behaviour: asEBehaviours,
        declaration: *const ::std::os::raw::c_char,
        funcPointer: *const asSFuncPtr,
        callConv: asDWORD,
    ) -> ::std::os::raw::c_int;
*/
    pub fn Script_String(str: *const ::std::os::raw::c_char) -> *mut ScriptString;
);

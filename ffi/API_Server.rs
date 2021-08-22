dynamic_ffi!(ServerApi, 

    pub fn Cl_RunClientScript(
        cl: *mut Critter,
        func_name: *const ::std::os::raw::c_char,
        p0: ::std::os::raw::c_int,
        p1: ::std::os::raw::c_int,
        p2: ::std::os::raw::c_int,
        p3: *const ::std::os::raw::c_char,
        p4: *const uint,
        p4_size: size_t,
    ) -> bool;

    pub fn Global_RunCritterScript(
        cr: *mut Critter,
        script_name: *const ::std::os::raw::c_char,
        p0: ::std::os::raw::c_int,
        p1: ::std::os::raw::c_int,
        p2: ::std::os::raw::c_int,
        p3_raw: *const ::std::os::raw::c_char,
        p4_ptr: *const uint,
        p4_size: size_t,
    ) -> bool;

    pub fn Global_GetCritter(crid: uint) -> *mut Critter;

    pub fn Global_GetMsgStr(lang: size_t, textMsg: size_t, strNum: uint) -> *mut ScriptString;

    pub fn Item_GetLexems(item: *mut Item) -> *mut ScriptString;

    pub fn ConstantsManager_GetValue(
        collection: size_t,
        string: *mut ScriptString,
    ) -> ::std::os::raw::c_int;

    pub fn Server_Statistics() -> *const ServerStatistics;

    pub fn Timer_GameTick() -> uint;

    pub fn Timer_FastTick() -> uint;
);

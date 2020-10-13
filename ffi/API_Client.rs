dynamic_ffi!(ClientApi, 

    pub fn Net_SendRunScript(
        unsafe_: bool,
        func_name: *const ::std::os::raw::c_char,
        p0: ::std::os::raw::c_int,
        p1: ::std::os::raw::c_int,
        p2: ::std::os::raw::c_int,
        p3: *const ::std::os::raw::c_char,
        p4: *const uint,
        p4_size: size_t,
    );

    pub fn Client_AnimGetCurSpr(anim_id: uint) -> uint;

    pub fn HexMngr_GetDrawTree() -> *mut Sprites;

    pub fn Sprites_InsertSprite(
        sprites: *mut Sprites,
        draw_order: ::std::os::raw::c_int,
        hx: ::std::os::raw::c_int,
        hy: ::std::os::raw::c_int,
        cut: ::std::os::raw::c_int,
        x: ::std::os::raw::c_int,
        y: ::std::os::raw::c_int,
        id: uint,
        id_ptr: *mut uint,
        ox: *mut ::std::os::raw::c_short,
        oy: *mut ::std::os::raw::c_short,
        alpha: *mut uchar,
        callback: *mut bool,
    ) -> *mut Sprite;

    pub fn Field_ChangeTile(
        field: *mut Field,
        anim: *mut AnyFrames,
        ox: ::std::os::raw::c_short,
        oy: ::std::os::raw::c_short,
        layer: uchar,
        is_roof: bool,
    );

    pub fn ResMngr_GetAnim(
        name_hash: uint,
        dir: ::std::os::raw::c_int,
        res_type: ::std::os::raw::c_int,
        filter_nearest: bool,
    ) -> *mut AnyFrames;

    pub fn HexMngr_GetAllItems_ScriptArray(items: *mut ScriptArray) -> size_t;

    pub fn HexMngr_GetScreenHexes(
        sx: *mut ::std::os::raw::c_int,
        sy: *mut ::std::os::raw::c_int,
    ) -> bool;

    pub fn HexMngr_GetHexCurrentPosition(
        hx: ushort,
        hy: ushort,
        x: *mut ::std::os::raw::c_int,
        y: *mut ::std::os::raw::c_int,
    ) -> bool;
);

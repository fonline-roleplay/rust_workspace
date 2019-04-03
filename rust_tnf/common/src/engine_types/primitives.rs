pub type int8 = ::std::os::raw::c_char;
pub type uint8 = ::std::os::raw::c_uchar;
pub type int16 = ::std::os::raw::c_short;
pub type uint16 = ::std::os::raw::c_ushort;
pub type uint64 = u64;
pub type int64 = i64;

pub type ulong = ::std::os::raw::c_ulong;
pub type ushort = ::std::os::raw::c_ushort;
pub type uint = ::std::os::raw::c_uint;
pub type int = ::std::os::raw::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SyncObj {
    pub CurMngr: *const ::std::os::raw::c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Spinlock {
    pub Locker: ::std::os::raw::c_long,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct stlp_std_pair<_T1, _T2> {
    pub first: _T1,
    pub second: _T2,
    pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_T1>>,
    pub _phantom_1: ::std::marker::PhantomData<::std::cell::UnsafeCell<_T2>>,
}

#[derive(Debug)]
pub enum stlp_std_allocator {}

pub type Uint16Pair = stlp_std_pair<uint16, uint16>;
pub type IntVec = stlp_std_vector<::std::os::raw::c_int, stlp_std_allocator>;
pub type Uint16Vec = stlp_std_vector<uint16, stlp_std_allocator>;
pub type Uint16PairVec = stlp_std_vector<Uint16Pair, stlp_std_allocator>;

#[repr(C)]
#[derive(Debug)]
pub struct stlp_std_vector<_Tp, _Alloc> {
    //pub _base: stlp_std_priv___Vector_base<_Tp, _Alloc>,
    //pub _phantom_0: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Tp>>,
    //pub _phantom_1: ::std::marker::PhantomData<::std::cell::UnsafeCell<_Alloc>>,
    start: usize,
    end: usize,
    reserved: usize,
    _phantom_0: ::std::marker::PhantomData<_Tp>,
    _phantom_1: ::std::marker::PhantomData<_Alloc>,
}

#[repr(C)]
pub struct stlp_std_set([u8; 24]);
pub type UintSet = stlp_std_set;

#[repr(C)]
pub struct stlp_std_string([u8; 24]);
#[repr(C)]
pub struct ScriptString__bindgen_vtable(::std::os::raw::c_void);
#[repr(C)]
pub struct ScriptString {
    pub vtable_: *const ScriptString__bindgen_vtable,
    pub buffer: stlp_std_string,
    pub refCount: ::std::os::raw::c_int,
}

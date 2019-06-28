pub type asDWORD = ::std::os::raw::c_ulong;
pub type asBYTE = ::std::os::raw::c_uchar;

#[repr(C)]
pub struct ScriptArray__bindgen_vtable(::std::os::raw::c_void);
#[repr(C)]
#[derive(Debug)]
pub struct ScriptArray {
    pub vtable_: *const ScriptArray__bindgen_vtable,
    pub refCount: ::std::os::raw::c_int,
    pub gcFlag: bool,
    pub objType: *mut asIObjectType,
    pub buffer: *mut ScriptArray_ArrayBuffer,
    pub elementSize: ::std::os::raw::c_int,
    pub cmpFuncId: ::std::os::raw::c_int,
    pub eqFuncId: ::std::os::raw::c_int,
    pub subTypeId: ::std::os::raw::c_int,
}

impl ScriptArray {
    pub fn buffer(&self) -> &[u8] {
        unsafe {
            let buf = &*self.buffer;
            let len = self.elementSize as usize * buf.numElements as usize;
            let slice = buf.data.get_unchecked(0..len);
            slice
        }
    }
    pub fn cast<T: 'static + Copy + Sync>(&self) -> Option<&[T]> {
        let buf = unsafe { &*self.buffer };
        let data = buf.data.as_ptr();

        let size = ::std::mem::size_of::<T>();
        let align = ::std::mem::align_of::<T>();

        if size != self.elementSize as usize || (data as usize) % align != 0 {
            return None;
        }
        let array: &[T] =
            unsafe { std::slice::from_raw_parts(data as *const T, buf.numElements as usize) };
        Some(array)
    }
    pub fn cast_struct<T: 'static + Copy + Sync>(&self) -> Option<&[T]> {
        let buf = self.buffer();
        let ptr = buf.as_ptr();

        let size = ::std::mem::size_of::<T>();
        let align = ::std::mem::align_of::<T>();

        if buf.len() % size != 0 || (ptr as usize) % align != 0 {
            return None;
        }
        let array: &[T] = unsafe { std::slice::from_raw_parts(ptr as *const T, buf.len() / size) };
        Some(array)
    }
    pub unsafe fn cast_pointer<T>(&self) -> Option<&[Option<&mut T>]> {
        let buf = unsafe { &*self.buffer };
        let data = buf.data.as_ptr();

        let size = ::std::mem::size_of::<*mut T>();
        let align = ::std::mem::align_of::<*mut T>();

        if size != self.elementSize as usize || (data as usize) % align != 0 {
            return None;
        }
        let array: &[Option<&mut T>] =
            std::slice::from_raw_parts(data as *const Option<&mut T>, buf.numElements as usize);
        Some(array)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ScriptArray_ArrayBuffer {
    pub numElements: asDWORD,
    pub data: [asBYTE; 1usize],
}

#[repr(C)]
pub struct asIObjectType__bindgen_vtable(::std::os::raw::c_void);
#[repr(C)]
#[derive(Debug)]
pub struct asIObjectType {
    pub vtable_: *const asIObjectType__bindgen_vtable,
}

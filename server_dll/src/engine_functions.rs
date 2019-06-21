use std::io::Write;
use tnf_common::engine_types::critter::Critter;
use tnf_common::engine_types::{ScriptArray, ScriptString};

const PTR_OFFSET: usize = 0x00400000;

const CL_RUN_CLIENT_SCRIPT: &[u8] = b"?Cl_RunClientScript@SScriptFunc@FOServer@@SAXPAVCritter@@AAVScriptString@@HHHPAV4@PAVScriptArray@@@Z";
type FnRunClientScript = unsafe extern "C" fn(
    &Critter,
    &ScriptString,
    i32,
    i32,
    i32,
    *const ScriptString,
    *const ScriptArray,
);

const GLOBAL_GET_CRITTER: &[u8] = b"?Global_GetCritter@SScriptFunc@FOServer@@SAPAVCritter@@I@Z";
type FnGetCritter = unsafe extern "C" fn(u32) -> *mut Critter;

struct FuncPointers {
    get_critter: FnGetCritter,
    run_client_script: FnRunClientScript,
}

static mut FUNC_POINTERS: Option<FuncPointers> = None;
static mut COMPAT_POINTERS: Option<ASCompat> = None;

#[repr(C)]
#[derive(Clone)]
struct ASCompat {
    new_script_string: fn(*const u8, usize) -> *mut ScriptString,
    release_script_string: fn(*mut ScriptString),
}

pub struct ScriptStringBox {
    inner: *mut ScriptString,
}

impl ScriptStringBox {
    fn new(str: &str) -> Self {
        let inner = unsafe {
            let func = COMPAT_POINTERS
                .as_ref()
                .expect("Compat pointers")
                .new_script_string;
            func(str.as_ptr(), str.len())
        };
        ScriptStringBox { inner }
    }
}

impl Drop for ScriptStringBox {
    fn drop(&mut self) {
        unsafe {
            let func = COMPAT_POINTERS
                .as_ref()
                .expect("Compat pointers")
                .release_script_string;
            func(self.inner);
        }
    }
}

impl AsRef<ScriptString> for ScriptStringBox {
    fn as_ref(&self) -> &ScriptString {
        unsafe { std::mem::transmute(self.inner) }
    }
}

pub fn init(compat: usize) {
    unsafe {
        FUNC_POINTERS = Some(
            load_func_pointers()
                .expect("pdb error")
                .expect("Can't load all function pointers"),
        );

        let ptr = compat as *const () as *const ASCompat;
        COMPAT_POINTERS = Some((&*ptr).clone());
    }
}

pub fn get_critter<'a>(id: u32) -> Option<&'a mut Critter> {
    unsafe { std::mem::transmute((FUNC_POINTERS.as_ref().expect("Func pointers").get_critter)(id)) }
}

pub fn run_client_script<'a>(cr: &mut Critter, func: &str, p0: i32, p1: i32, p2: i32) {
    use std::ptr::null;
    let string = ScriptStringBox::new(func);
    unsafe {
        (FUNC_POINTERS
            .as_ref()
            .expect("Func pointers")
            .run_client_script)(cr, string.as_ref(), p0, p1, p2, null(), null())
    }
}

/*
#[derive(Debug)]
enum GetPtrError {
    FileOpen(std::io::Error),
}
*/

fn load_func_pointers() -> pdb::Result<Option<FuncPointers>> {
    let file = std::fs::File::open("FOnlineServer.pdb")?; //.map_err(GetPtrError::FileOpen())?;
    let mut pdb = pdb::PDB::open(file)?;
    let symbol_table = pdb.global_symbols()?;
    let address_map = pdb.address_map()?;

    macro_rules! transmute_fn {
        ($($fn_static:ident: $fn_const:expr),+) => {
            FuncPointers{
                $(
                    $fn_static: {
                        if let Some(ptr) = get_ptr(&symbol_table, &address_map, $fn_const)? {
                            unsafe {
                                 std::mem::transmute(ptr as usize + PTR_OFFSET)
                            }
                        } else {
                            return Ok(None);
                        }
                    },
                )+
            }
        };
    }
    Ok(Some(transmute_fn! {
        get_critter: GLOBAL_GET_CRITTER,
        run_client_script: CL_RUN_CLIENT_SCRIPT
    }))
}

fn get_ptr(
    symbol_table: &pdb::SymbolTable,
    address_map: &pdb::AddressMap,
    func_name: &[u8],
) -> pdb::Result<Option<u32>> {
    let mut symbols = symbol_table.iter();
    use pdb::FallibleIterator;
    while let Some(symbol) = symbols.next()? {
        match symbol.parse()? {
            pdb::SymbolData::PublicSymbol(data) => {
                let name = symbol.name()?.as_bytes();
                if name == func_name {
                    if let Some(ptr) = data.offset.to_rva(&address_map) {
                        return Ok(Some(ptr.into()));
                    } else {
                        return Ok(None);
                    }
                }
            }
            //pdb::SymbolData::DataSymbol(data) => {
            //    print_row(data.offset, "data", symbol.name()?);
            //}
            //pdb::SymbolData::Procedure(data) => {
            //
            //}
            _ => {
                // ignore everything else
            }
        }
    }
    Ok(None)
}

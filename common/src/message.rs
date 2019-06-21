#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ServerDllToWeb {
    PlayerConnected(u32),
    PlayerAuth(u32),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ServerWebToDll {
    UpdateCharLeaf { id: u32, ver: u32, secret: u32 },
    SendKeyToPlayer(u32, [u32; 3]),
    Nop,
}

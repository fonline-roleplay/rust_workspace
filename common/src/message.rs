#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ServerDllToWeb {
    PlayerConnected(u32),
    PlayerAuth(u32),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ServerWebToDll {
    UpdateClientAvatar(u32, u32),
    SendKeyToPlayer(u32, [u32;3]),
}

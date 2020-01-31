use path::

fn convert<R: std::io::Read>(reader: &mut R) {
    let ext = Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or(Error::NoFileExtension)?;
}

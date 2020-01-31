#[cfg(feature = "generate")]
mod generate {
    #[cfg(feature = "r357")]
    const PATH_OUT: &str = "../src/generated/r357/";
    #[cfg(feature = "r476")]
    const PATH_OUT: &str = "../src/generated/r476/";

    use bindgen::{builder, CodegenConfig};

    fn generate(server: bool, types: &[&str], prefix: &str, file: &str, opaque: bool) {
        if opaque {
            generate_custom(server, types, types, &[], prefix, file);
        } else {
            generate_custom(server, types, &[], &[], prefix, file);
        }
    }
    fn generate_custom(
        server: bool,
        types: &[&str],
        opaque: &[&str],
        blacklist: &[&str],
        prefix: &str,
        file: &str,
    ) {
        #[cfg(feature = "r357")]
        let args = ["-xc++", "-m32", "-I../input/r357/StlPort", "-D_M_IX86"];
        #[cfg(feature = "r476")]
        let args = ["-xc++", "-m32"];

        #[cfg(feature = "r357")]
        let input = "../input/r357/fo.h";
        #[cfg(feature = "r476")]
        let input = "../input/r476/fo.h";

        let mut builder = builder()
            .clang_args(&args)
            .header(input)
            .raw_line("#[allow(unused_imports)] use super::*;\n")
            .whitelist_recursively(false)
            .with_codegen_config(CodegenConfig::TYPES);

        if server {
            builder = builder.clang_arg("-D__SERVER");
        } else {
            builder = builder.clang_arg("-D__CLIENT");
        }

        for ty in types {
            builder = builder.whitelist_type(ty);
        }
        for ty in opaque {
            builder = builder.opaque_type(ty);
        }
        for ty in blacklist {
            builder = builder.blacklist_type(ty);
        }
        let bindings = builder.generate().unwrap();
        bindings
            .write_to_file(format!("{}{}{}", PATH_OUT, prefix, file))
            .unwrap();
    }

    fn generate_conditional(server: bool) {
        let prefix = if server { "server/" } else { "client/" };

        std::fs::create_dir_all(format!("{}{}", PATH_OUT, prefix)).unwrap();
        generate_custom(
            server,
            &[
                "Critter",
                "Client",
                "Npc",
                "NpcPlane",
                "CritterTimeEvent",
                "GlobalMapGroup",
                "CritterCl",
            ],
            if server {
                &["CritterCl"][..]
            } else {
                &[
                    "Critter",
                    "Client",
                    "Npc",
                    "NpcPlane",
                    "CritterTimeEvent",
                    "GlobalMapGroup",
                ][..]
            },
            &[],
            prefix,
            "critter.rs",
        );

        generate(server, &["Item", "ProtoItem"], prefix, "item.rs", false);

        let map_types = [
            "Location",
            "Map",
            "ProtoMap",
            "SceneryToClient",
            "MapObject",
            "ProtoLocation",
            "MapEntire",
        ];
        let map_opaque = if server { &[][..] } else { &map_types[..] };
        generate_custom(
            server,
            &map_types,
            map_opaque,
            &["ProtoMap_TileVec"],
            prefix,
            "map.rs",
        );
        generate(
            server,
            &["Sprite", "SpriteInfo", "SpriteAnim"],
            prefix,
            "sprite.rs",
            server,
        );
        if server {
            generate_custom(
                server,
                &["Field", "Field_Tile"],
                &["Field", "Field_Tile"],
                &["Field_TileVec"],
                prefix,
                "field.rs",
            );
        } else {
            generate_custom(
                server,
                &["Field", "Field_Tile"],
                &[],
                &["Field_TileVec"],
                prefix,
                "field.rs",
            );
        }
        generate(
            server,
            &["GameOptions", "CritterMutual", "CritterType"],
            prefix,
            "state.rs",
            false,
        );
    }

    pub fn start() {
        std::fs::create_dir_all(PATH_OUT).unwrap();
        generate_conditional(true);
        generate_conditional(false);

        let opaque_types = ["Spinlock", "SyncObj", "Mutex"];
        generate_custom(
            true,
            &opaque_types,
            &opaque_types,
            &["ScriptArray_ArrayBuffer"],
            "",
            "opaque.rs",
        );

        let number_types = [
            "uint8", "uint16", "uint", "uint64", "int8", "int16", "int", "int64",
        ];
        generate(true, &number_types, "", "num.rs", false);
        generate(
            true,
            &[
                "ScriptString",
                "ScriptArray",
                "asIObjectType",
                "ArrayBuffer",
                "asDWORD",
                "asBYTE",
            ],
            "",
            "angelscript.rs",
            false,
        );
    }
}

fn main() {
    #[cfg(feature = "generate")]
    generate::start();
    #[cfg(not(feature = "generate"))]
    std::process::exit(1);
}

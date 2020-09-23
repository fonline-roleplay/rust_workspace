use fo_data::{Converter, FoData};

fn main() {
    let converter = FoData::init("../../../CL4RP", "../../../test_assets/COLOR.PAL")
        .unwrap()
        .into_retriever();
    let image = converter.get_png("art/tiles/BLD2010.FRM").unwrap();
    std::fs::write("../../../test_assets/output/BLD2010.png", image.data).unwrap();
}

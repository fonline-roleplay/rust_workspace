use fo_data::FoData;

fn main() {
    let fo_data = FoData::init("../../CL4RP", "../../test_assets/COLOR.PAL").unwrap();
    let image = fo_data.get_image("art/tiles/BLD2010.FRM").unwrap();
    std::fs::write("../../test_assets/output/BLD2010.png", image.data).unwrap();
}

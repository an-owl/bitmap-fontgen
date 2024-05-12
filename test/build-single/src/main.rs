use std::path::Path;

fn main() {
    let test_char: char = std::env::var("FONTGEN_TEST_CHAR").map(|v| v.chars().next().expect("FONTGEN_TEST_CHAR was null")).unwrap_or('a');

    let f = bitmap_fontgen::Font::from(&font_map::FONT);
    assert_eq!(f.weights().len(),1,"Incorrect number of font weights Expected 1 found {}",f.weights().len());
    assert_eq!(f.weights()[0],"Medium".into());
    assert_eq!(f.sizes().len(),1,"Incorrect number of sizes found: Expected 1 found {}",f.sizes().len());
    assert_eq!(f.sizes()[0].0,(16,32).into());

    let bdf = bdf::open(Path::new(env!("FONT_ORIGIN"))).expect("Failed to open bdf");

    let origin_glyph = bdf.glyphs().get(&test_char).expect(&format!("Failed to read glyph from {}. Is it supported by the font? If not set \"FONTGEN_TEST_CHAR\" to one that is", env!("FONT_ORIGIN")));
    let glyph = f.get("Medium".into(),(16,32).into(),'a').expect(&format!("Couldnt get glyph for '{}'",test_char));

    let mut buff = Vec::new();
    buff.resize((glyph.size().height * glyph.size().width) as usize,false);
    glyph.convert(|b| b,&mut buff);

    for (i, ((_,o),n)) in origin_glyph.pixels().zip(buff).enumerate(){ // origin, new
        assert_eq!(o,n,"Bitmaps did not match at px {i}")
    }
}

mod font_map {
    pub static FONT: bitmap_fontgen::ConstFontMap = include!(env!("FONT_FILE"));
}
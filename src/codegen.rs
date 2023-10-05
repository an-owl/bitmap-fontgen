use std::io::BufWriter;
use super::*;
use std::path::PathBuf;

struct GenMeta {
    name: String,
    weight: GenWeight,
    size: FontSize,
}

#[derive(Clone)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
struct GenWeight {
    data: String
}

impl PhfHash for GenWeight {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        for i in self.data.chars() {
            state.write_u32(i as u32);
        }
    }
}

impl PhfBorrow<GenWeight> for GenWeight {
    fn borrow(&self) -> &GenWeight {
        self
    }
}


impl FmtConst for GenWeight {
    fn fmt_const(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"::{name} {{data: {s}}}",name = std::any::type_name::<FontWeight>(),s = self.data)
    }
}

/// Generates the given fonts into the target file
pub fn gen_font<T: std::io::Write>(files: Vec<PathBuf>, target: &mut T) {

    for i in &files {
        assert!(i.exists(),"File not found: {}",i.display())
    }
    let face = bdf::open(&files[0]).unwrap().name().to_string();

    let mut internal_weight_map: std::collections::BTreeMap<GenWeight, Vec<(phf_codegen::Map<char>, GenMeta)>> = std::collections::BTreeMap::new();
    for i in &files {
        let t = compile_file(i,&face);
        if let Some(arr) = internal_weight_map.get_mut(&t.1.weight) {
            arr.push(t)
        } else {
            internal_weight_map.insert(t.1.weight.clone(),vec![t]);
        }
    }

    let mut weight_map = phf_codegen::Map::new();
    for (weight, fonts) in internal_weight_map {
        let mut size_map = phf_codegen::Map::new();
        for (i,m) in fonts {
            size_map.entry(m.size,&i.build().to_string());
        }
        weight_map.entry(weight,&size_map.build().to_string());
    }

    write!(target,"{}",weight_map.build()).unwrap();
}

fn get_weight(file: &PathBuf) -> String {
    // this looks like a mess but it's simple.
    // the file is a bunch of key value pairs
    // This opens the file and iterates over each line to find "FONT"
    // The value is divided by hyphens '-'. the weight should be the 3rd value in here. (this also accounts for the keys index)
    let s = String::from_utf8_lossy(&std::fs::read(file).unwrap()).to_string();
    for i in s.lines() {
        if i.starts_with("FONT") {
            return i.split("-").nth(3).unwrap().to_string();
        }
    }

    panic!("Unable to parse {}", file.display())

}

fn compile_file(file: &PathBuf,name: &str) -> (phf_codegen::Map<char>,GenMeta) {

    // gather metadata
    let weight = get_weight(file);
    let font = bdf::open(file).unwrap();
    assert_eq!(name, font.name(), "Multiple faces found {} & {} did not match", name, font.name());

    let m = GenMeta {
        name: name.to_string(),
        weight: GenWeight{data: weight},
        size: FontSize { width: font.size().x, height: font.size().y },
    };

    let mut map = phf_codegen::Map::new();

    // iterate over chars
    for (c,g) in font.glyphs() {

        let mut pxarr = Vec::<u8>::new();
        pxarr.resize((m.size.width*m.size.height) as usize/8,0);

        // iterate over pixels set one bits when necessary
        for (i,(_,p)) in g.pixels().enumerate() {
            let byte = i/8;
            let bit = 1%8;
            if p {
                pxarr[byte] |= 1 << bit;
            }
        }
        map.entry(*c,&format!("&{:?}",&*pxarr));
    }
    (map,m)
}
use super::*;
use std::path::PathBuf;

struct GenMeta {
    _name: String,
    weight: GenWeight,
    size: FontSize,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct GenWeight {
    data: String,
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
        write!(
            f,
            "::{name} {{inner: \"{s}\"}}",
            name = std::any::type_name::<FontWeight>(),
            s = self.data
        )
    }
}

/// Generates the given fonts into the target file
pub fn gen_font<T: std::io::Write>(files: Vec<PathBuf>, target: &mut T) {
    for i in &files {
        assert!(i.exists(), "File not found: {}", i.display())
    }
    let face = get_face(bdf::open(&files[0]).unwrap().name());

    let mut internal_weight_map: std::collections::BTreeMap<
        GenWeight,
        Vec<(phf_codegen::Map<char>, GenMeta)>,
    > = std::collections::BTreeMap::new();
    for i in &files {
        let t = compile_file(i, &face);
        if let Some(arr) = internal_weight_map.get_mut(&t.1.weight) {
            arr.push(t)
        } else {
            internal_weight_map.insert(t.1.weight.clone(), vec![t]);
        }
    }

    let mut weight_map = phf_codegen::Map::new();
    for (weight, fonts) in internal_weight_map {
        let mut size_map = phf_codegen::Map::new();
        for (mut i, m) in fonts {
            size_map.entry(m.size, &i.phf_path("fontgen::phf").build().to_string());
        }
        weight_map.entry(
            weight,
            &size_map.phf_path("fontgen::phf").build().to_string(),
        );
    }

    write!(target, "{}", weight_map.phf_path("fontgen::phf").build()).unwrap();
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

fn get_face(name: &str) -> String {
    name.split('-').nth(2).expect(&format!("Found invalid name format")).to_string()
}

fn compile_file(file: &PathBuf, tgt_name: &str) -> (phf_codegen::Map<char>, GenMeta) {
    // gather metadata
    let weight = get_weight(file);
    let font = bdf::open(file).unwrap();
    let name = get_face(font.name());
    assert_eq!(
        tgt_name,
        name,
        "Multiple faces found {} & {} did not match",
        tgt_name,
        name
    );

    let m = GenMeta {
        _name: name.to_string(),
        weight: GenWeight { data: weight },
        size: FontSize {
            width: font.bounds().width,
            height: font.bounds().height,
        },
    };

    let mut map = phf_codegen::Map::new();

    // iterate over chars
    for (c, g) in font.glyphs() {
        let mut pxarr = Vec::<u8>::new();

        // todo watch https://github.com/rust-lang/rust/issues/88581 and use div_ciel
        let pixarr_len = {
            if (m.size.width * m.size.height) as usize % 8 != 0 {
                ((m.size.width * m.size.height) as usize / 8) + 1
            } else {
                (m.size.width * m.size.height) as usize / 8
            }
        };

        pxarr.resize(pixarr_len, 0);

        // iterate over pixels set one bits when necessary
        for (i, (_, p)) in g.pixels().enumerate() {
            let byte = i / 8;
            let bit = i % 8;
            if p {
                pxarr[byte] |= 1 << bit;
            }
        }
        map.entry(*c, &format!("&{:?}", &*pxarr));
    }
    (map, m)
}

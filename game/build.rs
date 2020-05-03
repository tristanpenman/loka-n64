use n64_math::Color;
use png;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::{io::BufReader, path::Path};
use std::fs::{self, File};

struct Image {
    width: i32,
    height: i32,
    data: Vec<u8>,
}

fn load_png(path: &Path) -> Result<Image, Box<dyn Error>> {

    let file = File::open(path).map_err(|e| format!("Unable to open {:?}: {}", path, e))?;
    let decoder = png::Decoder::new(file);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    if info.color_type != png::ColorType::RGBA
        || info.bit_depth != png::BitDepth::Eight
        || info.buffer_size() != (4 * info.width * info.height) as usize
    {
        return Err("Image format not supported!")?;
    }

    Ok(Image {
        width: info.width as i32,
        height: info.height as i32,
        data: buf,
    })
}

fn parse_textures(out_dir: &str) -> Result<(), Box<dyn Error>> {
    let mut res = String::new();

    for path in fs::read_dir("textures")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| path.extension() == Some(OsStr::new("png")))
    {
        if let Some(name) = path.file_stem().map(|n| n.to_string_lossy()) {
            let image = load_png(path.as_path())?;

            let out_path = path.canonicalize()?.with_extension("ntex");

            let mut out_image = Vec::with_capacity((2 * image.width * image.height) as usize);

            for pixel in image.data.chunks(4) {
                let color = Color::from_bytes(pixel.try_into()?);
                out_image.extend(&color.value().to_le_bytes());
            }

            fs::write(&out_path, &out_image)?;

            res.push_str(&format!(
                "pub static {name}: StaticTexture = StaticTexture::from_static({width}, {height}, include_bytes!({path:?}));\n",
                name = name.to_uppercase(),
                width = image.width,
                height = image.height,
                path = out_path
            ));

            println!("rerun-if-changed={}", path.to_string_lossy());
        }
    }

    fs::write(Path::new(out_dir).join("texture_includes.rs"), res)?;

    Ok(())
}

fn parse_maps(out_dir: &str) -> Result<(), Box<dyn Error>> {
    let mut res = String::new();

    for path in fs::read_dir("maps")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| path.extension() == Some(OsStr::new("tmx")))
    {
        let file = File::open(&path).map_err(|e| format!("Unable to open {:?}: {}", path, e))?;
        let reader = BufReader::new(file);
        let map = tiled::parse_with_path(reader, &env::current_dir()?.join("maps"))?;
        
        println!("{:#?}", map.object_groups);

        println!("rerun-if-changed={}", path.to_string_lossy());
    }

    fs::write(Path::new(out_dir).join("map_includes.rs"), res)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;

    parse_textures(&out_dir)?;
    parse_maps(&out_dir)?;

    Ok(())
}

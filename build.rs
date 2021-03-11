use anyhow::*;
use glob::glob;
use std::fs::{read_to_string, write};
use std::path::PathBuf;

struct ShaderData {
    src: String,
    src_path: PathBuf,
    spv_path: PathBuf,
    kind: shaderc::ShaderKind,
}

impl ShaderData {
    pub fn load(src_path: PathBuf) -> Result<Self> {
        println!("2");
        // Try to get the extension of the file
        let extension = src_path
            // Takes in `PathBuf` and tries to return the extension of the file
            .extension()
            // If there's an error, return it with the message "file has no extension" (keep in mind `?` operator)
            .context("File has no extension")?
            // Convert our `OsStr` to `str`
            .to_str()
            // Again, return error with some context message if we have to
            .context("Extension cannot be converted to &str")?;
        // Get the kind of shader we're reading based off of the extension
        let kind = match extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            // `bail!` is basically `.context()` in a macro
            // (somepath)`.display()` returns the path but safe for printing without unicode characters
            _ => bail!("Unsupported shader: {}", src_path.display()),
        };

        // Get the contents of the shader file ofc
        let src = read_to_string(src_path.clone())?;
        // Set the file extension to name.(vert | frag | comp).spv
        // `with_extension` replaces extension to the file
        let spv_path = src_path.with_extension(format!("{}.spv", extension));

        Ok(Self {
            src,
            src_path,
            spv_path,
            kind,
        })
    }
}

// Remember we have `main` return `Result<()>` so we're able to use the `?` operator to auto return errors
fn main() -> Result<()> {
    // Collect all shaders recursively within /src/
    // Remember glob comes from `glob` pkg
    let mut shader_paths = [
        glob("./src/**/*.vert")?,
        glob("./src/**/*.frag")?,
        glob("./src/**/*.comp")?,
    ];

    // This could be parallelized
    // TODO fkin do not know closures ffs ffsfsfsfsf
    // Basically looks like we're iterating over each path, and for each path creating `shaderData`
    let shaders = shader_paths
        .iter_mut()
        .flatten()
        .map(|glob_result| ShaderData::load(glob_result?))
        // TODO have no idea what these `collect` statements do lol
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    // Init new shaderc compiler to compile the string
    // Compiler compiles GLSL into SPIR-V modules
    // let mut compiler = shaderc::Compiler::new().unwrap();
    // So compile both the vertex and fragment shader form GLSL into spirv
    let mut compiler = shaderc::Compiler::new().context("Unable to create shader compiler")?;

    // This can't be parallelized. The [shaderc::Compiler] is not
    // thread safe. Also, it creates a lot of resources. You could
    // spawn multiple processes to handle this, but it would probably
    // be better just to only compile shaders that have been changed
    // recently.
    for shader in shaders {
        // This tells cargo to rerun this script if something in /src/ changes.
        // TODO whoa cool wtf
        println!(
            "cargo:rerun-if-changed={}",
            shader.src_path.as_os_str().to_str().unwrap()
        );

        // Now we can basically just spread our `ShaderData` here to compile into SPIRV
        // note the `?` operator at the very end lol
        let compiled = compiler.compile_into_spirv(
            // Content of shader
            &shader.src,
            // Kind of shader
            shader.kind,
            // Path of file as `&str`
            &shader.src_path.to_str().unwrap(),
            "main",
            None,
        )?;
        // Write file to the designated `spv` bath as binary
        write(shader.spv_path, compiled.as_binary_u8())?;
    }

    Ok(())
}

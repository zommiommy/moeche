use std::{io::prelude::*, collections::HashMap};
use std::fs::File;
use std::path::Path;
use zip::write::{ZipWriter, FileOptions};
use sha2::{Sha256, Digest};
use base64ct::{Base64, Encoding};
use std::hash::{Hash, Hasher};
use regex::Regex;
use std::convert::TryFrom;
use goblin::{error, Object};
use scroll::{Pread, Pwrite, SizeWith};

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

macro_rules! file_options {
    () => {{
        FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .unix_permissions(0o644)
    }};
    ($permissions:expr) => {{
        FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .unix_permissions($permissions)
    }};
}

macro_rules! dir_options {
    () => {{
        FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .unix_permissions(0o755)
    }};
    ($permissions:expr) => {{
        FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .unix_permissions($permissions)
    }};
}

/// Metadata that will be in the RECORD file
struct FileMetaData {
    path: String,
    hash: String,
    size: usize,
}

/// Binary libraries that the crate depends on
struct DepLib {
    sha256: [u8; 32],
    path: String,
    name: String,
    content: Vec<u8>,
}

impl Hash for DepLib {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // we already know the hash so we don't need to hash everything else
        self.sha256.hash(state);
    }
}

pub enum PythonTag {
    GenericPython,
    CPython,
    IronPython,
    PyPy,
    Jython,
}

impl<'a> TryFrom<&'a str> for PythonTag {
    type Error = String;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let value = value.trim().to_lowercase();
        match value.as_str() {
            "py" => Ok(PythonTag::GenericPython),
            "cp" => Ok(PythonTag::CPython),
            "ip" => Ok(PythonTag::IronPython),
            "pp" => Ok(PythonTag::PyPy),
            "jy" => Ok(PythonTag::Jython),
            _ => Err(format!(
                "The normalized python tag '{}' is not a valid one.",
                value,
            ))
        }
    }
}

impl std::fmt::Display for PythonTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PythonTag::GenericPython => "py",
            PythonTag::CPython       => "cp",
            PythonTag::IronPython    => "ip",
            PythonTag::PyPy          => "pp",
            PythonTag::Jython        => "jy",
        })
    }
}

pub struct WheelSettings {
    // data from https://peps.python.org/pep-0425/

    /// The name of the packager
    lib_name: String,
    /// The version of the package
    version: String,
    /// Which interpreter the wheel is for
    python_tag: PythonTag,
    /// The abi compatability tag
    abi_tag: String,
    /// The platform compatability tag
    platform_tag: String,
    /// python 3.x
    python_minor_version: usize,

    /// Where to write the wheel
    dst_wheel_folder_path: String, 
    /// Where is the readme of the package
    readme_path: String,
    
    /// Dependancies on other python packages
    requires_dist: Vec<String>, 
    /// Keywords
    keywords: Vec<String>,
    /// Authors
    authors: Vec<String>,
    /// Authors emails
    author_emails: Vec<String>,
    /// License
    license: String,
    /// Project url
    project_url: Option<String>,
}

impl WheelSettings {
    pub fn validate(&mut self) -> Result<(), String> {
        // https://packaging.python.org/en/latest/specifications/binary-distribution-format/

        // validate the libname
        // PEP 503
        let libname_regex = Regex::new(r"[a-z0-9\-]+").unwrap();
        self.lib_name = self.lib_name.trim()
            .to_ascii_lowercase();
        let libname_regex_normalize = Regex::new(r"[-._]+").unwrap();
        self.lib_name = libname_regex_normalize.replace_all(&self.lib_name, "-").to_string();
        if !libname_regex.is_match(&self.lib_name) {
            return Err(format!(
                "The normalized lib name '{}' does not respet PEP 503 format.",
                self.lib_name,
            ));
        }

        // validate the version
        // PEP 440 396 345
        // TODO!: this doesn't currently support the local version segment
        let version_regex = Regex::new(r"[\d+\!]\d+(.\d+)*[{a|b|rc}\d+][.post\d|_+][.dev\d+]").unwrap();
        self.version = self.version.trim().to_ascii_lowercase();
        if !version_regex.is_match(&self.version) {
            return Err(format!(
                "The normalized version '{}' does not respet PEP 440 496 345 format.",
                self.lib_name,
            ));
        }

        // TODO!: validate the abi_tag
        // PEP 384 ??

        Ok(())
    }
}

pub struct WheelBuilder {
    /// The path to the wheel we are writing to
    dst_file: String,

    settings: WheelSettings,
    /// an handle to the file where we are writing to
    writer: ZipWriter<File>,
    /// Data to build the RECORD file
    hashes: Vec<FileMetaData>,

    // paths in the zip
    package_path: String,
    metadata_path: String,
    bin_libs_path: String,
    /// The hasmap of file hash -> in package path of the libraries,
    /// This is used to deduplicate the libs so we can save memory
    bin_libs: HashMap<[u8; 32], String>,

    /// if finish was already called
    is_built: bool,
}

impl WheelBuilder {
    pub fn get_file_path(&self) -> &str {
        self.dst_file.as_str()
    }

    pub fn new(mut settings: WheelSettings) -> Result<Self, String> {
        // check that the user isn't an asshole
        settings.validate()?;

        // defined here: https://peps.python.org/pep-0425/
        let wheel_name = format!(
            "{libname}-3.{python_minor_version}-{python_tag}-{abi_tag}-{platform_tag}.whl",
            libname=settings.lib_name,
            python_minor_version=settings.python_minor_version,
            python_tag =settings.python_tag,
            abi_tag=settings.abi_tag,
            platform_tag=settings.platform_tag,
        );
        
        let dst_file = Path::new(&settings.dst_wheel_folder_path).join(wheel_name);
        // create the wheel file
        let file = File::create(&dst_file)
            .map_err(|e| format!(
                "Could not create the file '{}'. The error is: '{}'",
                dst_file.display(), e,
            ))?;
        // and initialize a zip writer on it
        let writer = zip::ZipWriter::new(file);

        // compute once the path for the subfolders
        let package_path  = settings.lib_name.clone();
        let metadata_path = format!("{}-{}.dist-info", settings.lib_name, settings.version);
        let bin_libs_path = format!("{}-libs", settings.lib_name);

        Ok(WheelBuilder{
            dst_file: dst_file.display().to_string(),

            settings,
            writer,
            hashes: vec![],

            package_path,
            metadata_path,
            bin_libs_path,

            bin_libs: HashMap::new(),

            is_built: false,
        })
    }

    fn handle_elf<R: Read>(&mut self, mut file: R) -> Result<Vec<u8>, String> {
        // Read the whole elf to a Vec<u8> because goblin need it ffs
        // the alternative is to mmap it into memory but that increases complexity
        // a lot so it will be done in the future
        let mut buffer = Vec::with_capacity(4096);
        let mut chunk = Vec::with_capacity(4096);
        loop {
            let n_of_bytes_read = file.read(&mut chunk)
                .map_err(|e| format!(
                    "Could not read file ELF file.",
                ))?;

            if n_of_bytes_read == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..n_of_bytes_read]);
        }

        // parse the elf
        let obj = Object::parse(&buffer).map_err(|e| {
            format!("Error parsing an ELF. The error is '{}'", e)
        })?;

        let result = match obj {
            Object::Elf(mut elf) => {
                // set the rpath so that it will load local libs
                let rpath = format!("$ORIGIN/../{}", self.bin_libs_path);
                elf.rpaths = vec![&rpath];

                // get the dependancies, take note of them and patch the imports
                println!("{:?}", elf.libraries);

                //let mut result = vec![];
                //result.pwrite_with::<goblin::elf::Elf>(elf, 0, scroll::NATIVE)
                unimplemented!()
            }
            _ => panic!("This shouldn't happen, it's a file with the signature of an ELF but it's not a valid elf."),
        };

        Ok(result)
    }

    pub fn add_package_file<R: Read + Seek, P: AsRef<Path>>(&mut self, mut file: R, dst_path: P) -> Result<(), String> {
        let dst_path = dst_path.as_ref();

        let mut magic = [0_u8; 4];
        file.read_exact(&mut magic).map_err(|e| {format!(
            "Could not read magic of file with dst: '{}'",
            dst_path.display(),
        )})?;

        // rewind the magic
        file.seek(std::io::SeekFrom::Start(0))
            .map_err(|e| {format!(
                "Could not seek back the given file with dst: '{}'. The error is '{}'",
                dst_path.display(), e,
            )})?;

        if magic == [0x7F, b'E', b'L', b'F'] {
            let patched_elf = self.handle_elf(file)?;
            let file_ref: &[u8] = patched_elf.as_ref();
            return self.add_file(
                file_ref, 
                dst_path.display().to_string()
            );
        } else {
            return self.add_file(
                file, 
                dst_path.display().to_string()
            );
        }
    }

    fn add_file<R: Read>(&mut self, mut file: R, dst_path: String) -> Result<(), String> {
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 4096];
        let mut file_size = 0;

        // start writing a new file
        self.writer.start_file(
            &dst_path, 
            file_options!(),
            ).map_err(|e| format!(concat!(
                "Could not start new file in the wheel at path: '{}'. ",
                "The error is: '{}'"),
                self.dst_file, e,
            ))?;

        // read as much as possible the file, while keeping track of its size
        // and computing its sha256 hash
        loop {
            // read a chunk of the file
            let n_of_bytes_read = file.read(&mut buffer)
                .map_err(|e| format!(
                    "Could not read file for: '{}'. The error is '{:?}'",
                    dst_path, e,
                ))?;

            if n_of_bytes_read == 0 {
                break;
            }
            // update the file size
            file_size += n_of_bytes_read;

            // update the hash
            hasher.update(&buffer[..n_of_bytes_read]);

            // write inside the zip
            self.writer.write_all(&buffer[..n_of_bytes_read])
                .map_err(|e| format!(
                    "Could not write inside the zip file with path: '{}'. The error is '{}'",
                    self.dst_file, e,
                ))?;

        }

        // convert the hash to hex
        let hash = hasher.finalize();
        let hex_hash = Base64::encode_string(&hash).trim_end_matches("=").to_string();

        self.hashes.push(FileMetaData { 
            path: dst_path, 
            hash: hex_hash, 
            size: file_size, 
        }); 

        Ok(())
    }

    /// create the RECORD file
    fn create_record_file(&self) -> String {
        let mut record_file = String::with_capacity(4096);
        for metadata in self.hashes.iter() {
            record_file.push_str(&format!(
                "{}={},{}\n",
                metadata.path, metadata.hash, metadata.size,
            ));
        }

        record_file.push_str(&format!(
            "{},,", Path::new(&self.metadata_path).join(Path::new("RECORD")).display()
        ));
        record_file
    }

    /// create the WHEEL file
    /// the fields are explained here:
    /// https://packaging.python.org/en/latest/specifications/binary-distribution-format/
    fn create_wheel_file(&self) -> String {
        let mut result = String::with_capacity(4096);
        result.push_str("Wheel-Version: 1.0\n");
        result.push_str(&format!("Generator: {} ({}) \n", CRATE_NAME, CRATE_VERSION));
        result.push_str("Root-Is-Purelib: false\n");

        // wheel’s expanded compatibility tags;
        // TODO!:
        //result.push_str(&format!("Tag: {}\n", platform_compatability_tag));
        
        result
    }

    /// create the METADATA file
    fn create_metadata_file(&self) -> Result<String, String> {
        let mut result = String::with_capacity(4096);
        result.push_str("Metadata-Version: 2.1\n");
        result.push_str(&format!("Name: {}\n", self.settings.lib_name));
        result.push_str(&format!("Version: {}\n", self.settings.version));
        result.push_str(&format!("License: {}\n", self.settings.license));
        result.push_str(&format!("Requires-Python: >=3.{}\n", self.settings.python_minor_version));
        
        for req in self.settings.requires_dist.iter() {
            result.push_str(&format!("Requires-Dist: {}\n", req));
        }
        if !self.settings.keywords.is_empty() {
            result.push_str(&format!("Keywords: {}\n", self.settings.keywords.join(",")));
        }
        if !self.settings.authors.is_empty() {
            result.push_str(&format!("Authors: {}\n", self.settings.authors.join(", ")));
        }
        if !self.settings.author_emails.is_empty() {
            result.push_str(&format!("Author-email: {}\n", self.settings.author_emails.join(", ")));
        }

        if let Some(project_url) = self.settings.project_url.as_ref() {
            result.push_str(&format!("Project-URL: Source Code, {}\n", project_url));
        }

        result.push_str("Description-Content-Type: text/markdown; charset=UTF-8; variant=GFM\n");

        // empty line for start of readme
        result.push('\n');


        result.push_str(&std::fs::read_to_string(&self.settings.readme_path)
            .map_err(|e| format!(
                "Cannot read redme at path: '{}'. The error is: '{}'",
                self.settings.readme_path, e,
            ))?);

        Ok(result)
    }


    /// inner implementation, this is so we can call this method in the drop
    /// so we won't forget it acciedentaly.
    /// The finish method still exists because this method could raise errors,
    /// so you can catch them by calling the finish method explicitely, otherwise
    /// the drop will call finish.unwrap()
    fn inner_finish(&mut self) -> Result<(), String> {
        if self.is_built {
           return Ok(()) 
        }

        // going to write .dist-info at the end of the file as recommended by
        // https://packaging.python.org/en/latest/specifications/binary-distribution-format/

        // create the metadata folder
        self.writer.add_directory(
            &self.metadata_path, 
            dir_options!()
        ).map_err(|e| format!(
            "Could not create the metadata folder inside the wheel: '{}'. The error is: '{}'",
            self.dst_file, e,
        ))?;

        self.add_file(
            self.create_wheel_file().as_bytes(),
            Path::new(&self.bin_libs_path).join("WHEEL").display().to_string(),
        )?;
        
        self.add_file(
            self.create_metadata_file()?.as_bytes(),
            Path::new(&self.bin_libs_path).join("METADATA").display().to_string(),
        )?;
        
        // Important! the RECORD file should **ALWAYS** be the last
        self.add_file(
            self.create_record_file().as_bytes(),
            Path::new(&self.bin_libs_path).join("RECORD").display().to_string(),
        )?;
        
        // consolidate the zip
        self.writer.finish()
        .map_err(|e|
            format!(concat!(
                "Could not finish writing the zip with path: '{}' .",
                " The error is '{}'",
            ), self.dst_file, e,
        ))?;
        self.is_built = true;
        Ok(())
    }

    /// Finish writing the metadata. This is automatically done by the drop.
    /// The problem is that writing to disk could always fail and the drop can't
    /// return an error so it will call unwrap. 
    /// So if you want to catch possible errors you should explicitely call 
    /// this method and handle the error.
    /// Calling this method will ensure that the drop will not panic.
    pub fn finish(mut self) -> Result<(), String> {
        self.inner_finish()
    }
}

impl Drop for WheelBuilder {
    fn drop(&mut self) {
        self.inner_finish().unwrap();
    }
}
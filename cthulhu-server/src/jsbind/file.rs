use rquickjs::{class::Trace, Class, Object};

use std::{
    fs::Metadata,
    path::{Path, PathBuf},
    rc::Rc,
};

use rquickjs::{Ctx, Result};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

use super::{throw_js_err, to_js_err};
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
#[rquickjs::class(rename = "Metadata")]
#[derive(Debug, Trace, Clone)]
pub struct JsMetadata {
    #[qjs(skip_trace)]
    pub inner: Metadata,
}
#[rquickjs::methods(rename_all = "camelCase")]
impl JsMetadata {
    #[qjs(constructor)]
    pub fn new(ctx: rquickjs::Ctx<'_>) -> rquickjs::Result<Self> {
        Err(throw_js_err("Illegal constructor", ctx))
    }
    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }
    pub fn is_file(&self) -> bool {
        self.inner.is_file()
    }
    pub fn is_symlink(&self) -> bool {
        self.inner.is_symlink()
    }
    pub fn last_access_time(&self) -> u64 {
        #[cfg(target_os = "windows")]
        {
            self.inner.last_access_time()
        }

        #[cfg(target_os = "linux")]
        {
            self.inner.st_atime() as _
        }
    }
    pub fn last_write_time(&self) -> u64 {
        #[cfg(target_os = "windows")]
        {
            self.inner.last_write_time()
        }

        #[cfg(target_os = "linux")]
        {
            self.inner.st_atime() as _
        }
    }
    pub fn creation_time(&self) -> u64 {
        #[cfg(target_os = "windows")]
        {
            self.inner.creation_time()
        }

        #[cfg(target_os = "linux")]
        {
            0
        }
    }
    pub fn file_size(&self) -> u64 {
        #[cfg(target_os = "windows")]
        {
            self.inner.file_size()
        }

        #[cfg(target_os = "linux")]
        {
            self.inner.st_size() as _
        }
    }
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
}

#[rquickjs::class(rename = "Path")]
#[derive(Debug, Trace, Clone)]
pub struct JsPath {
    #[qjs(skip_trace)]
    pub inner: PathBuf,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsPath {
    #[qjs(constructor)]
    pub fn new<'js>(path: String) -> Self {
        let path = Path::new(&path).to_path_buf();
        Self { inner: path }
    }
    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }
    pub fn is_file(&self) -> bool {
        self.inner.is_file()
    }
    pub fn is_symlink(&self) -> bool {
        self.inner.is_symlink()
    }
    pub fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }
    pub fn is_relative(&self) -> bool {
        self.inner.is_relative()
    }
    pub fn extension(&self) -> Option<String> {
        self.inner
            .extension()
            .map(|v| v.to_string_lossy().to_string())
    }
    pub fn file_name(&self) -> Option<String> {
        self.inner
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
    }
    pub fn file_stem(&self) -> Option<String> {
        self.inner
            .file_stem()
            .map(|v| v.to_string_lossy().to_string())
    }
    pub fn metadata(&self) -> Option<JsMetadata> {
        self.inner.metadata().ok().map(|inner| JsMetadata { inner })
    }
    pub fn to_path(&self, base: String, ctx: Ctx<'_>) -> Result<Self> {
        let path = self.inner.as_path();
        let path = relative_path::RelativePath::from_path(path)
            .map_err(|e| to_js_err(e, ctx))?
            .to_path(base);
        Ok(Self { inner: path })
    }
    pub fn to_logical_path(&self, base: String, ctx: Ctx<'_>) -> Result<Self> {
        let path = self.inner.as_path();
        let path = relative_path::RelativePath::from_path(path)
            .map_err(|e| to_js_err(e, ctx))?
            .to_logical_path(base);
        Ok(Self { inner: path })
    }

    pub fn exists(&self, _ctx: Ctx<'_>) -> bool {
        self.inner.exists()
    }
    pub fn to_string(&self) -> String {
        self.inner.to_str().unwrap().to_string()
    }
}
#[rquickjs::class(rename = "File")]
#[derive(Debug, Trace, Clone)]
pub struct JsFile {
    #[qjs(skip_trace)]
    pub file: Rc<Mutex<File>>,
    #[qjs(get)]
    pub path: JsPath,
    #[qjs(get)]
    pub metadata: JsMetadata,
}
#[rquickjs::methods]
impl JsFile {
    #[qjs(constructor)]
    pub fn new<'js>(
        path: JsPath,
        cfg: rquickjs::function::Opt<Object<'js>>,
        ctx: Ctx<'js>,
    ) -> Result<Self> {
        if !path.is_file() {
            return Err(throw_js_err("path is not a file", ctx));
        }
        let opt = cfg
            .0
            .map(|v| -> Result<std::fs::OpenOptions> {
                let read = v.get::<_, Option<bool>>("read")?.unwrap_or(false);
                let write = v.get::<_, Option<bool>>("write")?.unwrap_or(false);
                let create = v.get::<_, Option<bool>>("create")?.unwrap_or(false);
                let create_new = v.get::<_, Option<bool>>("createNew")?.unwrap_or(false);
                let append = v.get::<_, Option<bool>>("append")?.unwrap_or(false);
                let truncate = v.get::<_, Option<bool>>("truncate")?.unwrap_or(false);
                let mut opt = std::fs::OpenOptions::new();
                opt.read(read)
                    .write(write)
                    .append(append)
                    .create(create)
                    .create_new(create_new)
                    .truncate(truncate);
                Ok(opt)
            })
            .unwrap_or(Ok({
                let mut opt = std::fs::OpenOptions::new();
                opt.read(true);
                opt
            }))
            .map_err(|e| to_js_err(e, ctx.clone()))?;

        let f = opt
            .open(&path.inner)
            .map_err(|e| to_js_err(e, ctx.clone()))?;
        let metadata = f.metadata().map_err(|e| to_js_err(e, ctx))?;
        let f = Rc::new(Mutex::new(tokio::fs::File::from_std(f)));
        return Ok(Self {
            file: f,
            metadata: JsMetadata { inner: metadata },
            path,
        });
    }

    #[qjs(rename = "writeBytes")]
    pub async fn write_bytes<'js>(&self, buf: Vec<u8>, ctx: Ctx<'js>) -> Result<()> {
        let mut file = self.file.lock().await;
        match file.write_all(&buf).await {
            Ok(v) => Ok(v),
            Err(e) => {
                let s = e.to_string();
                Err(ctx.throw(
                    rquickjs::String::from_str(ctx.clone(), s.as_str())
                        .unwrap()
                        .into(),
                ))
            }
        }
    }
    #[qjs(rename = "readBytes")]
    pub async fn read_bytes<'js>(&self, ctx: Ctx<'js>) -> Result<Vec<u8>> {
        let mut file = self.file.lock().await;
        let mut buf: Vec<u8> = vec![];
        if let Err(e) = file.read_to_end(&mut buf).await {
            let s = e.to_string();
            return Err(ctx.throw(
                rquickjs::String::from_str(ctx.clone(), s.as_str())
                    .unwrap()
                    .into(),
            ));
        }
        Ok(buf)
    }

    #[qjs(rename = "toString")]
    pub fn to_string(&self) -> String {
        format!(r#"File(path="{}")"#, &self.path.to_string())
    }
}

pub fn init_def(_id: &str, ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();
    Class::<JsFile>::define(&globals)?;
    Class::<JsPath>::define(&globals)?;
    Class::<JsMetadata>::define(&globals)?;
    Ok(())
}

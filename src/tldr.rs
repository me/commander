use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use failure::Error;
use flate2::read::GzDecoder;
use log::debug;
use pulldown_cmark::{Event, Parser, Tag};
use reqwest::{Client, Proxy};
use tar::Archive;

use super::commands;

const ARCHIVE_URL: &str = "https://github.com/tldr-pages/tldr/archive/master.tar.gz";
const DIR: &str = "tldr";
const COMMON: &str = "common";
const MD: &str = "md";

pub struct Updater<'a> {
  pub url: Option<&'a str>,
  pub path: OsString,
}

impl<'a> Updater<'a> {
  pub fn new(url: Option<&'a str>, path: OsString) -> Updater<'a> {
    Updater {
      url: url,
      path: PathBuf::from(&path).join(DIR).into_os_string(),
    }
  }

  pub fn update(&self) -> Result<(), Error> {
    let bytes: Vec<u8> = self.download()?;
    let mut archive = self.decompress(&bytes[..]);
    fs::create_dir_all(&self.path)?;
    self.parse_files(&mut archive)?;
    Ok(())
  }

  fn download(&self) -> Result<Vec<u8>, Error> {
    let mut builder = Client::builder();
    if let Ok(ref host) = env::var("HTTP_PROXY") {
      if let Ok(proxy) = Proxy::http(host) {
        builder = builder.proxy(proxy);
      }
    }
    if let Ok(ref host) = env::var("HTTPS_PROXY") {
      if let Ok(proxy) = Proxy::https(host) {
        builder = builder.proxy(proxy);
      }
    }
    let client = builder.build().unwrap_or_else(|_| Client::new());
    let url = self.url.unwrap_or(ARCHIVE_URL);
    let mut resp = client.get(url).send()?;
    let mut buf: Vec<u8> = vec![];
    let bytes_downloaded = resp.copy_to(&mut buf)?;
    debug!("{} bytes downloaded", bytes_downloaded);
    Ok(buf)
  }

  fn decompress<R: Read>(&self, reader: R) -> Archive<GzDecoder<R>> {
    Archive::new(GzDecoder::new(reader))
  }

  fn parse_files<R: Read>(&self, archive: &mut Archive<GzDecoder<R>>) -> Result<(), Error> {
    let mut common = Some(fs::File::create(
      PathBuf::from(&self.path)
        .join(COMMON)
        .with_extension(commands::EXTENSION),
    )?);
    let platform_name = Updater::get_os();
    let mut platform = match platform_name {
      Some(str) => Some(fs::File::create(
        PathBuf::from(&self.path)
          .join(str)
          .with_extension(commands::EXTENSION),
      )?),
      _ => None,
    };

    for file in archive.entries().unwrap() {
      let mut file = file?;

      let path = file.header().path()?;
      if path.extension().unwrap_or(&OsString::from("")) != MD {
        continue;
      }

      let pages = path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|p| p.to_str());
      // TODO: allow selecting locale
      match pages {
        Some("pages") => (),
        _ => continue,
      }

      let folder = path.parent().and_then(|p| p.file_name());
      if folder.is_none() {
        continue;
      }
      let folder = folder.unwrap();

      let writer;

      if folder.to_str() == Some("common") {
        writer = common.as_mut();
      } else if folder.to_str() == platform_name {
        writer = platform.as_mut();
      } else {
        writer = None;
      }
      if writer.is_none() {
        continue;
      }
      let writer = writer.unwrap();

      let mut s = String::new();
      file.read_to_string(&mut s)?;
      self.parse_file(&s, writer)?;
    }
    Ok(())
  }
  fn parse_file<W: Write>(&self, source: &str, writer: &mut W) -> Result<(), Error> {
    let parser = Parser::new(source);
    let mut descr = String::default();
    let mut cmd = String::default();
    let mut in_list_item = false;
    let mut in_command = false;
    let mut in_blockquote = false;
    for event in parser {
      match event {
        Event::Start(tag) => match tag {
          Tag::BlockQuote => in_blockquote = true,
          Tag::Code => in_command = !in_list_item && !in_blockquote,
          Tag::Item => in_list_item = true,
          _ => (),
        },
        Event::End(tag) => match tag {
          Tag::BlockQuote => {
            in_blockquote = false;
          }
          Tag::Item => {
            in_list_item = false;
          }
          Tag::Code => {
            if !in_command {
              continue;
            }
            if !descr.is_empty() {
              descr.truncate(descr.len() - 1);
            }
            if !cmd.is_empty() {
              write!(writer, "{} ## {}\n", cmd, descr)?;
            }
            descr.truncate(0);
            cmd.truncate(0);
            in_command = false;
          }
          _ => (),
        },
        Event::Text(t) => {
          if in_list_item {
            descr.push_str(&t);
          } else if in_command {
            cmd.push_str(&t);
          }
        }
        _ => (),
      }
    }
    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn get_os() -> Option<&'static str> {
    Some("linux")
  }

  #[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
  ))]
  fn get_os() -> Option<&'static str> {
    Some("osx")
  }

  #[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
  )))]
  fn get_os() -> Option<&'static str> {
    None
  }
}

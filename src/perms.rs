// Automatically generated rust module for 'perms.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use std::collections::HashMap;
type KVMap<K, V> = HashMap<K, V>;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct permission {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
}

impl<'a> MessageRead<'a> for permission {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.mode = r.read_uint32(bytes)?,
                Ok(16) => msg.uid = r.read_uint32(bytes)?,
                Ok(24) => msg.gid = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for permission {
    fn get_size(&self) -> usize {
        0
        + if self.mode == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.mode) as u64) }
        + if self.uid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.uid) as u64) }
        + if self.gid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.gid) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.mode != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.mode))?; }
        if self.uid != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.uid))?; }
        if self.gid != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.gid))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct permissions<'a> {
    pub permission: KVMap<Cow<'a, str>, permission>,
}

impl<'a> MessageRead<'a> for permissions<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_message::<permission>(bytes)?))?;
                    msg.permission.insert(key, value);
                }
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for permissions<'a> {
    fn get_size(&self) -> usize {
        0
        + self.permission.iter().map(|(k, v)| 1 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_len((v).get_size()))).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for (k, v) in self.permission.iter() { w.write_with_tag(10, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_len((v).get_size()), 10, |w| w.write_string(&**k), 18, |w| w.write_message(v)))?; }
        Ok(())
    }
}


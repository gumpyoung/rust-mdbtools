#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]
extern crate libc;
#[macro_use]
extern crate log;
use std::ffi::CString;
mod mdb;


pub type MapTableRow = std::collections::HashMap<String, String>;
pub type MapTable = Vec<MapTableRow>;
pub type MapDB = std::collections::HashMap<String, MapTable>;
pub struct MDB {
	handle : Option<*mut mdb::MdbHandle>
}

impl MDB{
	pub fn new(db_file : &str) -> MDB{
		let c_str_1 = CString::new(db_file).unwrap();
		info!("MDB::new: {}", db_file);
		unsafe{
			let handle = Some(mdb::mdb_open(c_str_1.as_ptr(), mdb::MDB_NOFLAGS));
			return MDB{handle: handle};
		}
	}
	pub fn to_map(&self) -> Option<MapDB>{
		let res = None;
		//Read tables
		unsafe{
			mdb::mdb_dump_catalog(self.handle.unwrap(), mdb::MDB_TABLE);
		}
		return res;
	} 	
}

#[test]
fn test(){
	let db_file = "/Users/steven/rusttest/avd.accdb";
	let db = MDB::new(db_file);
	db.to_map();
	
}



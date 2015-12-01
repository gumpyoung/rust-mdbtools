#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]
extern crate libc;
#[macro_use]
extern crate log;
extern crate glib_sys as glib_ffi;
extern crate rustc_serialize;
use std::ffi::CString;
use std::ffi::CStr;
use std::collections::HashMap;
mod mdb;

use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;

pub type MapTableRow = HashMap<String, String>;
pub type MapTable = Vec<MapTableRow>;
pub type MapDB = HashMap<String, MapTable>;
pub type TableRowData = Vec<String>;

type CatalogEntryPtr = *mut mdb::MdbCatalogEntry;
type ColumnPtr = *mut mdb::MdbColumn;
type TableDefPtr = *mut mdb::MdbTableDef;


pub struct MDBValueBindingItem{
	value_ptr : *mut libc::c_void,
	len_ptr : *mut i32
}

pub struct MDBTable{
	table_def_ptr : *mut mdb::MdbTableDef,
	name : String,
	rows : Vec<TableRowData>,
	column_names : Vec<String>,
	bindings : Vec<MDBValueBindingItem>
}


pub struct MDB<'a>{
	db_file : &'a String,
	handle : *mut mdb::MdbHandle,
	tables : Vec<MDBTable>
}

impl <'a> MDBTable{
	fn new(catalog_entry_ptr : CatalogEntryPtr) -> Option<MDBTable>{
		let mut res = None;
		let table_def_ptr = unsafe{mdb::mdb_read_table(catalog_entry_ptr)};
		if !table_def_ptr.is_null(){
			let rows = Vec::new();
			let column_names = Vec::new();
			let bindings = Vec::new();
			// Name
			let catalog_name = unsafe{(*catalog_entry_ptr).object_name};
			let cstr = unsafe{CStr::from_ptr(&catalog_name as *const i8)};
			match cstr.to_str() {
				Ok(str_name) => {
					if str_name.starts_with("MSys"){
						println!(">>> Ignored table -> {}", str_name);
					}
					else{
						println!(">>> Create object for table -> {}", str_name);
						res = Some(MDBTable{
							table_def_ptr : table_def_ptr,
							name : str_name.to_string(),
							rows: rows,
							column_names : column_names,
							bindings : bindings
						});
					}
				},
				Err(error) => {
					println!("---Parse table name error: {}", error);
				}
			}
		}
		return res;
	}

	fn load_column_names(&mut self) {
		self.column_names.clear();
		unsafe{
			mdb::mdb_read_columns(self.table_def_ptr);
			mdb::mdb_read_indices(self.table_def_ptr);
			mdb::mdb_rewind_table(self.table_def_ptr);
			let ptr_array = (*self.table_def_ptr).columns as *const mdb::GPtrArray;
			for i in 0..(*ptr_array).len {
				//println!("---col[{}]", i);
				let col_ptr = *(*ptr_array).pdata.offset(i as isize) as ColumnPtr;
				let col_name = (*col_ptr).name;
				let cstr = CStr::from_ptr(&col_name as *const i8);
				let str_result = cstr.to_str();
				match str_result {
					Ok(str_name) => {
						//println!("---Column Name: {}", str_name);
						self.column_names.push(str_name.to_string());
					},
					Err(error) => {
						println!("---Parse column name Error: {}", error);
					}
				}
			}
		}
	}

	fn bind(&mut self) {
    	//println!("------- Enter bind");
    	self.bindings.clear();
		for ci in 0..self.column_names.len(){
			unsafe{
				let mut bind_item = MDBValueBindingItem::new();
				mdb::mdb_bind_column(self.table_def_ptr,
					 (ci+1) as i32,
					 bind_item.value_ptr(),
					 bind_item.len_ptr()
				 );
				self.bindings.push(bind_item);
			 }
		}
    	//println!("------- Leave bind");
	}

	fn load_rows(&mut self) {
    	println!("------- Enter load_rows");
		self.rows.clear();
		unsafe{
			self.bind();
			let num_of_rows = (*self.table_def_ptr).num_rows;

            for _ in 0..num_of_rows{
                mdb::mdb_fetch_row(self.table_def_ptr);
				let mut row_data = Vec::new();
                for ci in 0..self.column_names.len() {
                	let str_val = self.bindings[ci].string_value();
                    row_data.push(str_val);
                }
                self.rows.push(row_data);
            }
		}
    	println!("------- Num of rows:{}", self.rows.len());
		
	}

	fn load(&mut self){
		self.load_column_names();
		self.load_rows();
	}

	fn to_map(&mut self) -> MapTable{
		let mut res = MapTable::new();
		for ri in 0..self.rows.len(){
			let mut row_data = MapTableRow::new();
			for ci in 0..self.column_names.len(){
				let col_name = self.column_names[ci].clone().to_string();
				row_data.insert(col_name, self.rows[ri][ci].clone());
			}
			res.push(row_data);
		}
		return res;
	}
}

impl MDBValueBindingItem{
	pub fn new() -> MDBValueBindingItem{
		const BINDING_SIZE : usize = 258;
		unsafe{
			let v_ptr = libc::calloc(BINDING_SIZE,1);
			let mut l_ptr = libc::calloc(4,1) as *mut i32;
			*l_ptr = BINDING_SIZE as i32;
			return MDBValueBindingItem{
				 value_ptr : v_ptr,
				 len_ptr: l_ptr};
		}
	}

	fn drop(&mut self){
		unsafe{
			libc::free(self.value_ptr);
			libc::free(self.len_ptr as *mut libc::c_void);
		}
	}


	pub fn value_ptr(&mut self) -> *mut libc::c_void{
		return self.value_ptr;
	}

	pub fn len_ptr(&mut self) -> *mut i32{
		return self.len_ptr;
	}

	pub fn string_value(& self) -> String{
		let res : String;
		unsafe{
	        //res = CStr::from_ptr(self.value_ptr as *const i8).to_string_lossy().into_owned();
	        res = CStr::from_ptr(self.value_ptr as *const i8).to_string_lossy().into_owned();
		}
		//println!("string_value {}", res);
		return res;

	}
}

impl <'a> MDB <'a> {
	pub fn new(db_file : &'a String) -> Option<MDB>{
		let handle = MDB::open_db(db_file);
		if handle.is_null(){
			return None;
		}
		else{
			return Some(MDB{
					db_file: db_file,
					handle: handle,
					tables : Vec::new()}
				);
		}
	}

	fn open_db(db_file : &String) -> *mut mdb::MdbHandle {
		let res: *mut mdb::MdbHandle;
		let c_str_1 = CString::new(&db_file[..]).unwrap();
		info!("MDB::new: {}", db_file);
		unsafe{
			res = mdb::mdb_open(c_str_1.as_ptr(), mdb::MDB_NOFLAGS);
		}
		return res;

	}


	pub fn load(& mut self) -> bool{
		let res = true;
		println!(">>> Read catalog");
		let p_handle = self.handle;
        let rv = unsafe{mdb::mdb_read_catalog(p_handle, mdb::MDB_TABLE)};
        if !rv.is_null(){
    		unsafe{
				let handle = *p_handle;
        		let ptr_array = handle.catalog as *const mdb::GPtrArray;
        		for i in 0..handle.num_catalog {
        			let catalog_ptr = *(*ptr_array).pdata.offset(i as isize) as CatalogEntryPtr;
					//println!(">>> Before Initialise table {}", i);
					let tbl = MDBTable::new(catalog_ptr);
					match tbl {
						None => {
							println!(">>> No table created for index {}", i);
						},
						Some(mut obj_tbl) => {
							obj_tbl.load();
							self.tables.push(obj_tbl);
						}
					}
					//println!(">>> After Initialise table {}", i);
				}
			}
        }
		return res;
	}
	pub fn to_map(&mut self) -> MapDB{
		let mut res = MapDB::new();
		for i in 0..self.tables.len(){
			res.insert(self.tables[i].name.clone(), self.tables[i].to_map());
		}
		return res;
	}
	
	pub fn to_json(&mut self) -> String {
		return json::as_pretty_json(&self.to_map()).to_string();
	}
}

#[test]
fn test(){
	let db_file = "/Users/steven/rusttest/avd.accdb".to_string();
	println!("Loading db_file: {}", db_file);
	let mut db = MDB::new(&db_file).unwrap();
	if db.load()
	{
		println!("Json: {}", db.to_json());
	}
	println!("Num of tables: {}", db.tables.len());
}



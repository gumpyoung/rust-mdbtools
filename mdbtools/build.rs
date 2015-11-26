extern crate gcc;
extern crate pkg_config;
use std::env;

fn compile_c_lib(
		inc_paths : &[&str],
		src_files : &[&str],
		output : &str){
	
	let mut cfg = gcc::Config::new();
	
	for inc_path in inc_paths {
		cfg.include(inc_path);
	}
		
	for src_file in src_files{
		cfg.file(src_file);
	}
	cfg.compile(output);
}

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();
		
	let src_files = ["mdbtools/src/libmdb/catalog.c",
        "mdbtools/src/libmdb/dump.c",
        "mdbtools/src/libmdb/iconv.c",
        "mdbtools/src/libmdb/like.c",
        "mdbtools/src/libmdb/options.c",
        "mdbtools/src/libmdb/sargs.c",
        "mdbtools/src/libmdb/table.c",
        "mdbtools/src/libmdb/write.c",
        "mdbtools/src/libmdb/backend.c",
        "mdbtools/src/libmdb/data.c",
        "mdbtools/src/libmdb/file.c",
        "mdbtools/src/libmdb/index.c",
        "mdbtools/src/libmdb/map.c",
        "mdbtools/src/libmdb/money.c",
        "mdbtools/src/libmdb/props.c",
        "mdbtools/src/libmdb/stats.c",
        "mdbtools/src/libmdb/worktable.c"
        ];	
        
        
    let inc_paths = [
    	"./mdbtools/include",
    	"/usr/local/Cellar/glib/2.46.2/include/glib-2.0"
    	];        

    compile_c_lib(&inc_paths,
    	  &src_files, 
    	  "libmdb.a");
    
	println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=mdb");
}
// 2022 Hiroyuki Ogasawara
// vim:ts=4 sw=4 noet:

use std::env;
mod mdtowiki;

fn	usage()
{
	println!( "mdtowiki v1.10 2022 Hiroyuki Ogasawara" );
	println!( "usage: mdtowiki [<options>] <input_file>" );
	println!( "option:" );
	println!( "  -l<type>     md,doku,puki" );
	println!( "  -s<type>     md,doku,puki,red,conf" );
	println!( "  -o<output_file>" );
	println!( "  --all" );
	println!( "  --dump" );
	println!( "ex.  mdtowiki -lmd readme.md -sdoku -odokuwiki.txt -spuki -opukiwiki.txt" );
	std::process::exit( 1 );
}


fn	main()
{
	let	mut	load_type= String::from("md");
	let	mut	save_type= String::from("doku");
	let	mut	input_file= String::new();
	let	mut	debug_dump= false;
	let	mut	save_list: Vec<(String,String)>= Vec::new();
	let	mut	all_flag= false;
	for arg in env::args().skip( 1 ) {
		if arg.starts_with( "-" ) {
			if arg.starts_with( "-l" ) {
				load_type= arg[2..].to_string();
			}else if arg.starts_with( "-s" ) {
				save_type= arg[2..].to_string();
			}else if arg.starts_with( "-o" ) {
				save_list.push( (save_type.clone(), arg[2..].to_string()) );
			}else if arg == "--dump" {
				debug_dump= true;
			}else if arg == "--all" {
				all_flag= true;
			}else{
				usage();
			}
		}else{
			input_file= arg;
		}
	}


	if input_file.is_empty() {
		usage();
	}


	println!( "load [{}]:  {}", load_type, input_file );
	let	document;
	match load_type.as_str() {
		"md" => {
			let	loader= mdtowiki::w_md::Decoder::new();
			document= loader.load( &input_file );
		},
		"doku" => {
			let	loader= mdtowiki::w_doku::Decoder::new();
			document= loader.load( &input_file );
		},
		"puki" => {
			let	loader= mdtowiki::w_puki::Decoder::new();
			document= loader.load( &input_file );
		},
		_ => {
			println!( "Unknown load type \"{}\"", load_type );
			std::process::exit( 1 );
		},
	}

	match document {
		Ok(doc) => {
			if debug_dump {
				doc.dump();
			}
			if all_flag {
				save_list.push( ("md".to_string(), "output.md".to_string()) );
				save_list.push( ("doku".to_string(), "output.doku".to_string()) );
				save_list.push( ("puki".to_string(), "output.puki".to_string()) );
				save_list.push( ("red".to_string(), "output.red".to_string()) );
				save_list.push( ("conf".to_string(), "output.conf".to_string()) );
			}
			for (save_type,output_file) in &save_list {
				println!( "save [{}]:  {}", save_type, output_file );
				match save_type.as_str() {
					"md" => {
						doc.save( &output_file, &mdtowiki::w_md::Encoder::new() ).unwrap();
					},
					"doku" => {
						doc.save( &output_file, &mdtowiki::w_doku::Encoder::new() ).unwrap();
					},
					"puki" => {
						doc.save( &output_file, &mdtowiki::w_puki::Encoder::new() ).unwrap();
					},
					"red" => {
						doc.save( &output_file, &mdtowiki::w_red::Encoder::new() ).unwrap();
					},
					"conf" => {
						doc.save( &output_file, &mdtowiki::w_conf::Encoder::new() ).unwrap();
					},
					_ => {
						usage();
					},
				}
			}
		},
		_ => {
			println!( "Load Error \"{}\"", input_file );
			std::process::exit( 1 );
		},
	}
}

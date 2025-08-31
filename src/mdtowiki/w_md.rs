// 2022 Hiroyuki Ogasawara
// vim:ts=4 sw=4 noet:

use std::fs;
use std::io::{self,BufRead};
use regex::{self,Regex};
use	lazy_static::lazy_static;

use super::element::*;
use super::document::*;


#[allow(dead_code)]
fn print_type<T>( _: &T )
{
    println!( "{} byte  \"{}\"",
            std::mem::size_of::<T>(),
            std::any::type_name::<T>() );
}


//=============================================================================

struct EMMarkPat {
	pat:  Regex,
	cmd1: &'static str,
	cmd2: &'static str,
}

pub fn	replace_md_tags( line: &str ) -> String
{
	lazy_static! {
		static ref	PAT_EM_ARRAY: Vec<EMMarkPat>= vec![
				EMMarkPat{ pat: Regex::new( r"^(.*\b)___([^_]+)___(\b.*)$" ).unwrap(),			cmd1: "\x07B3", cmd2: "\x07b3", },
				EMMarkPat{ pat: Regex::new( r"^(.*\b)__([^_]+)__(\b.*)$" ).unwrap(),			cmd1: "\x07B2", cmd2: "\x07b2", },
				EMMarkPat{ pat: Regex::new( r"^(.*\b)_([^_]+)_(\b.*)$" ).unwrap(),				cmd1: "\x07B2", cmd2: "\x07b1", },
				EMMarkPat{ pat: Regex::new( r"(^.*[^\\]|^)\*\*\*([^*]+)\*\*\*(.*)$" ).unwrap(),	cmd1: "\x07B3", cmd2: "\x07b3", },
				EMMarkPat{ pat: Regex::new( r"(^.*[^\\]|^)\*\*([^*]+)\*\*(.*)$" ).unwrap(),		cmd1: "\x07B2", cmd2: "\x07b2", },
				EMMarkPat{ pat: Regex::new( r"(^.*[^\\]|^)\*([^*]+)\*(.*)$" ).unwrap(),			cmd1: "\x07B1", cmd2: "\x07b1", },
				EMMarkPat{ pat: Regex::new( r"^(.*)~~([^~]+)~~(.*)$" ).unwrap(),				cmd1: "\x07D0", cmd2: "\x07d0", },
				EMMarkPat{ pat: Regex::new( r"^(.*)`([^`]+)`(.*)$" ).unwrap(),					cmd1: "\x07C0", cmd2: "\x07c0", },
			];
		static ref	PAT_LINK: Regex= Regex::new( r"^(.*)\[(.+)\]\((.+)\)(.*)$" ).unwrap();
	}
	let	mut	buffer= line.to_string();
	loop {
		let	mut	bfound= false;
		for emmark in PAT_EM_ARRAY.iter() {
			if let Some(v)= emmark.pat.captures( &buffer ) {
				let	mut buffer2= String::new();
				buffer2+= &v[1];
				buffer2+= emmark.cmd1;
				buffer2+= &v[2];
				buffer2+= emmark.cmd2;
				buffer2+= &v[3];
				buffer= buffer2;
				bfound= true;
				break;
			}
		}
		if let Some(v)= PAT_LINK.captures( &buffer ) {
			let	mut buffer2= String::new();
			buffer2+= &v[1];
			buffer2+= "\x07L0";
			buffer2+= &v[3];
			buffer2+= "\x07L1";
			buffer2+= &v[2];
			buffer2+= "\x07L2";
			buffer2+= &v[4];
			buffer= buffer2;
			bfound= true;
		}
		if !bfound {
			break;
		}
	}

	return	buffer;
}

pub fn	decode_from_md( line0: &str ) -> String
{
	let	line= &replace_md_tags( line0 );
	let	mut	buffer= String::new();
	let	mut	char_it= line.chars();
	loop {
		let	ch= char_it.next();
		match ch {
			Some('&') => {
				if char_it.as_str().starts_with( "lt;" ) {
					buffer+= "<";
					char_it.next();
					char_it.next();
					char_it.next();
				}else if char_it.as_str().starts_with( "gt;" ) {
					buffer+= ">";
					char_it.next();
					char_it.next();
					char_it.next();
				}else if char_it.as_str().starts_with( "amp;" ) {
					buffer+= "&";
					char_it.next();
					char_it.next();
					char_it.next();
					char_it.next();
				}else{
					buffer+= "&";
				}
			},
			Some('\\') => {
				if let	Some(c)= char_it.next() {
					match c {
						'*'|'_'|'\\'|'`'|'#'|'+'|'-'|'.'|'!'|'{'|'}'|'['|']'|'('|')'|'<'|'>' => {
							buffer+= &format!( "{}", c );
						},
						_ => {
							buffer+= &format!( "\\{}", c );
						},
					}
				}else{
					buffer+= "\\";
					return	buffer;
				}
			},
			Some(c) => {
				buffer+= &format!( "{}", c );
			},
			None => {
				return	buffer;
			},
		}
	}
}

pub fn	encode_to_md( line: &str ) -> String
{
	let	mut	buffer= String::new();
	let	mut	char_it= line.chars();
	loop {
		let	ch= char_it.next();
		match ch {
			Some('<') => {
				//buffer+= "&lt;";
				buffer+= "\\<";
			},
			Some('>') => {
				//buffer+= "&gt;";
				buffer+= "\\>";
			},
			/*Some('_') => {
				buffer+= "\\_";
			},*/
			Some('*') => {
				buffer+= "\\*";
			},
			Some('\x07') => {
				let	cmd0= char_it.next().unwrap();
				let	cmd1= char_it.next().unwrap();
				match cmd0 {
					'B'|'b' => {
						match cmd1 {
							'1' => {
								buffer+= "*";
							},
							'2' => {
								buffer+= "**";
							},
							_ => {
								buffer+= "***";
							},
						}
					},
					'D'|'d' => {
						buffer+= "~~";
					},
					'C'|'c' => {
						buffer+= "`";
					},
					'L' => {
						match cmd1 {
							'0' => {
								let	mut	url= "";
								let	mut	text= "";
								if let Some(upos)= char_it.as_str().find( '\x07' ) {
									url= &char_it.as_str()[..upos];
									for _ in 0..upos {
										char_it.next();
									}
									char_it.next();
									char_it.next();
									char_it.next();
									if let Some(tpos)= char_it.as_str().find( '\x07' ) {
										text= &char_it.as_str()[..tpos];
										for _ in 0..tpos {
											char_it.next();
										}
										char_it.next();
										char_it.next();
										char_it.next();
									}
								}
								buffer+= "[";
								buffer+= text;
								buffer+= "](";
								buffer+= url;
								buffer+= ")";
							},
							_ => {
							},
						}
					},
					_ => {
					},
				}
			},
			Some(c) => {
				buffer+= &format!( "{}", c );
			},
			None => {
				return	buffer;
			},
		}
	}
}




//=============================================================================

trait ElementGenerator {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>;
}


struct GenerateorContext {
	// list
	list_prev_indent: u32,
	list_nest: u32,
	list_indent: Vec<u32>,
	// pre
	pre_block: bool,
	pre_code: String,
	pre_data: String,
	// table
	table_block: bool,
	table_column: Vec<TableColumn>,
	table_pat: regex::Regex,
	table_alpat: regex::Regex,
}

impl GenerateorContext {
	fn	new() -> Self
	{
		GenerateorContext{
				list_prev_indent: 0,
				list_nest: 0,
				list_indent: vec![0;32],
				pre_block: false,
				pre_code: String::new(),
				pre_data: String::new(),
				table_block: false,
				table_column: Vec::new(),
				table_pat: regex::Regex::new( r"^\|(.*)\|\s*$" ).unwrap(),
				table_alpat: regex::Regex::new( r"^\s*([-:]+)\s*$" ).unwrap(),
			}
	}
	//-------------------------------------------------------------------------
	fn	indent_to_nest( &mut self, indent: u32 ) -> u32
	{
		let mut	nest;
		if indent == 0 {
			self.list_prev_indent= 0;
			self.list_nest= 0;
			self.list_indent[0]= 0;
			nest= 0;
		}else{
			if indent > self.list_prev_indent {
				self.list_nest+= 1;
				self.list_indent[self.list_nest as usize]= indent;
				self.list_prev_indent= indent;
				nest= self.list_nest;
			}else if indent < self.list_prev_indent {
				nest= 0;
				for ni in (1..self.list_nest+1).rev() {
					if indent >= self.list_indent[ni as usize] {
						nest= ni;
						self.list_nest= ni;
						break;
					}
				}
				self.list_prev_indent= indent;
			}else{
				nest= self.list_nest;
			}
		}
		return	nest;
	}
	//-------------------------------------------------------------------------
	fn	is_pre_block( &self ) -> bool
	{
		self.pre_block
	}
	fn	add_pre_block( &mut self, line: &str ) -> Option<Box<dyn Element>>
	{
		if line.starts_with( "```" ) {
			self.pre_block= false;
			return	Some( Box::new( PRETagElement{
							etype:	ElementType::PRETAG,
							text: 	(&self.pre_data).to_string(),
							code:	(&self.pre_code).to_string(),
						}));
		}
		//self.pre_data+= &decode_from_md_pre( &line );
		self.pre_data+= &line;
		self.pre_data+= "\n";
		return	None;
	}
	//-------------------------------------------------------------------------
	fn	is_table_block( &self ) -> bool
	{
		self.table_block
	}
	fn	add_table_block( &mut self, line: &str ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.table_pat.captures( line );
		if let Some(v)= pat_result {
			//println!( "TABLE={}", line );
			let	tline= &v[1];
			let	mut	params= tline.split( '|' ).peekable();
			let	alpat_result= self.table_alpat.captures( params.peek().unwrap() );
			if let Some(_)= alpat_result {
				//println!( "AlignLine" );
				for (i,param) in params.enumerate() {
					let	mut	align= ETableAlign::DEFAULT;
					let	tparam= &param.trim();
					if tparam.starts_with( ":" ) {
						if tparam.ends_with( ":" ) {
							align= ETableAlign::CENTER;
						}else{
							align= ETableAlign::LEFT;
						}
					}else if tparam.ends_with( ":" ) {
						align= ETableAlign::RIGHT;
					}
					if i < self.table_column[0].data.len() {
						self.table_column[0].data[i].align= align;
					}
				}
				for attr in &mut self.table_column[0].data {
					attr.header= true;
				}
			}else{
				let	mut	column= TableColumn::new();
				for (i,td) in params.enumerate() {
					let	mut	align= ETableAlign::DEFAULT;
					if self.table_column.len() > 0 && i < self.table_column[0].data.len() {
						align= self.table_column[0].data[i].align;
					}
					column.add( TableAttr{
							text: decode_from_md( td.trim() ),
							align: align,
							header: false,
						} );
				}
				self.table_column.push( column )
			}
		}else{
			self.table_block= false;
			return	Some( Box::new( TABLEElement{
							etype:	ElementType::TABLE,
							data:	self.table_column.to_vec(),
						}));
		}
		return	None;
	}
	//-------------------------------------------------------------------------
}

//-----------------------------------------------------------------------------

struct HTagGen {
	pat : regex::Regex,
}

impl HTagGen {
	fn	new() -> Self
	{
		HTagGen{
			pat: regex::Regex::new( r"^(#+)\s+(.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for HTagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			//println!( "{} {} {}", &v[1], &v[2], v[1].len() );
			return	Some( Box::new( HTagElement{
							etype:	ElementType::HTAG,
							title: 	v[2].to_string(),
							level:	v[1].len() as u32,
						}));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct LITagGen {
	pat_ul : regex::Regex,
	pat_ol : regex::Regex,
}

impl LITagGen {
	fn	new() -> Self
	{
		LITagGen{
			pat_ul: regex::Regex::new( r"^(\s*)[-+*]\s+(.*)$" ).unwrap(),
			pat_ol: regex::Regex::new( r"^(\s*)[0-9]+\.\s+(.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for LITagGen {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat_ul.captures( line );
		if let Some(v)= pat_result {
			//println!( "ul {} {} {}", &v[2], &v[1], v[1].len() );
			let	indent= v[1].len() as u32;
			let nest= context.indent_to_nest( indent );
			return	Some( Box::new( LITagElement{
							etype:	ElementType::ULTAG,
							text: 	decode_from_md( &v[2] ),
							indent:	indent,
							nest:	nest,
						}));
		}
		let	pat_result= self.pat_ol.captures( line );
		if let Some(v)= pat_result {
			//println!( "ol {} {} {}", &v[2], &v[1], v[1].len() );
			let	indent= v[1].len() as u32;
			let nest= context.indent_to_nest( indent );
			return	Some( Box::new( LITagElement{
							etype:	ElementType::OLTAG,
							text: 	decode_from_md( &v[2] ),
							indent:	indent,
							nest:	nest,
						}));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct PRETagGen {
	pat : regex::Regex,
}

impl PRETagGen {
	fn	new() -> Self
	{
		PRETagGen{
			pat: regex::Regex::new( r"^```(.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for PRETagGen {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			//println!( "pre {}", &v[1] );
			context.pre_block= true;
			context.pre_code= v[1].to_string();
			context.pre_data= String::new();
			return	Some( Box::new( NONEElement{} ));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct TABLEGen {
	pat : regex::Regex,
}

impl TABLEGen {
	fn	new() -> Self
	{
		TABLEGen{
			pat: regex::Regex::new( r"^\|.*\|\s*$" ).unwrap(),
		}
	}
}

impl ElementGenerator for TABLEGen {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(_)= pat_result {
			context.table_block= true;
			context.table_column= Vec::new();
			if let Some(e)= context.add_table_block( line ) {
				return	Some(e);
			}
			return	Some( Box::new( NONEElement{} ));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct HRTagGen {
	pat : regex::Regex,
}

impl HRTagGen {
	fn	new() -> Self
	{
		HRTagGen{
			pat: regex::Regex::new( r"^-\s*-(\s*-)+$" ).unwrap(),
		}
	}
}

impl ElementGenerator for HRTagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(_)= pat_result {
			return	Some( Box::new( HRTagElement{} ) );
		}
		return	None;
	}
}



//-----------------------------------------------------------------------------

pub struct Decoder {
	gen_table: Vec<Box<dyn ElementGenerator>>,
}

impl Decoder {
	pub fn new() -> Self
	{
		let	gen_table: Vec<Box<dyn ElementGenerator>>= vec![
			Box::new( HRTagGen::new() ),
			Box::new( HTagGen::new() ),
			Box::new( LITagGen::new() ),
			Box::new( PRETagGen::new() ),
			Box::new( TABLEGen::new() ),
		];
		Decoder{ gen_table: gen_table }
	}
	fn	find( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		for gen in &self.gen_table {
			let	element= gen.generate( line, context );
			if let Some(_)= element {
				return	element;
			}
		}
		return	None;
	}
	pub fn	load( &self, file_name: &str ) -> io::Result<Document>
	{
		let	file= fs::File::open( file_name )?;
		let	reader= io::BufReader::new( file );

		let	mut	page= Document::new();
		let	mut	context= GenerateorContext::new();

		for rline in reader.lines() {
			match rline {
				Ok(line) => {
					//let	line= decode_from_md( &line0 );
					if context.is_pre_block() {
						if let Some(e)= context.add_pre_block( &line ) {
							page.push( e );
						}
						continue;
					}else if context.is_table_block() {
						if let Some(e)= context.add_table_block( &line ) {
							page.push( e );
						}else{
							continue;
						}
					}
					{
						if let Some(e)= self.find( &line, &mut context ) {
							page.push( e );
						}else{
							page.push( Box::new(DataElement{ text: decode_from_md( &line ) }) );
						}
					}
				},
				Err(e) => {
					return	Err(e);
				},
			}
		}
		if context.is_pre_block() {
			if let Some(e)= context.add_pre_block( "" ) {
				page.push( e );
			}
		}else if context.is_table_block() {
			if let Some(e)= context.add_table_block( "" ) {
				page.push( e );
			}
		}
		Ok(page)
	}
}


//=============================================================================

trait	EncodeElement {
	fn	output( &self ) -> String;
}


//-----------------------------------------------------------------------------

impl	EncodeElement for NONEElement {
	fn	output( &self ) -> String
	{
		return	String::new();
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for HTagElement {
	fn	output( &self ) -> String
	{
		let	tag= '#'.to_string().repeat( self.level as usize );
		return	format!( "{} {}\n", tag, self.title );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for LITagElement {
	fn	output( &self ) -> String
	{
		let	mut	spaces= "".to_string();
		let	indent= (self.nest) * 2;
		if indent > 0 {
			spaces= ' '.to_string().repeat( indent as usize );
		}
		if self.etype == ElementType::ULTAG {
			return	format!( "{}- {}\n", spaces, encode_to_md( &self.text ) );
		}else{
			return	format!( "{}1. {}\n", spaces, encode_to_md( &self.text ) );
		}
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for PRETagElement {
	fn	output( &self ) -> String
	{
		return	format!( "```{}\n{}```\n", self.code, &self.text );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for TABLEElement {
	fn	output( &self ) -> String
	{
		let	mut	buffer= String::new();
		for column in &self.data {
			buffer+= "|";
			let mut	bheader= false;
			for attr in &column.data {
				buffer+= &format!( " {} |", encode_to_md( &attr.text ) );
				bheader= attr.header;
			}
			buffer+= "\n";
			if bheader {
				buffer+= "|";
				for attr in &column.data {
					match attr.align {
						ETableAlign::CENTER => {
							buffer+= ":----:|";
						},
						ETableAlign::LEFT => {
							buffer+= ":----|";
						},
						ETableAlign::RIGHT => {
							buffer+= "----:|";
						},
						_ => {
							buffer+= "-----|";
						},
					}
				}
				buffer+= "\n";
			}
		}
		return	buffer;
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for HRTagElement {
	fn	output( &self ) -> String
	{
		return	"----\n".to_string();
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for DataElement {
	fn	output( &self ) -> String
	{
		encode_to_md( &self.text ) + "\n"
	}
}


//-----------------------------------------------------------------------------

pub struct	Encoder {
}

impl	Encoder {
	pub	fn	new() -> Self
	{
		Self{}
	}
}

impl	TextEncoder for Encoder {
	fn	encode_single( &self, element: &Box<dyn Element> ) -> String
	{
		let	any= element.as_any();
		if let Some(e)= any.downcast_ref::<NONEElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<HTagElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<LITagElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<PRETagElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<TABLEElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<HRTagElement>() {
			return	e.output();
		}
		if let Some(e)= any.downcast_ref::<DataElement>() {
			return	e.output();
		}
		return	String::new();
	}
}


//-----------------------------------------------------------------------------




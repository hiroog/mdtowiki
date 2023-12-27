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

//=============================================================================

pub fn	replace_doku_tags( line: &str ) -> String
{
	lazy_static! {
		static ref	PAT_EM_ARRAY: Vec<EMMarkPat>= vec![
				EMMarkPat{ pat: Regex::new( r"^(.*)\*\*([^*]+)\*\*(.*)$" ).unwrap(),	cmd1: "\x07B2", cmd2: "\x07b2", },
				EMMarkPat{ pat: Regex::new( r"^(.*)//([^/]+)//(.*)$" ).unwrap(),		cmd1: "\x07B1", cmd2: "\x07b1", },
				EMMarkPat{ pat: Regex::new( r"^(.*)<del>([^/]+)</del>(.*)$" ).unwrap(),	cmd1: "\x07D0", cmd2: "\x07d0", },
			];
		static ref	PAT_LINK2: Regex= Regex::new( r"^(.*)\[\[([^|]+)\|(.*)\]\](.*)$" ).unwrap();
		static ref	PAT_LINK1: Regex= Regex::new( r"^(.*)\[\[(.+)\]\](.*)$" ).unwrap();
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
		if bfound {
			continue;
		}
		if let Some(v)= PAT_LINK2.captures( &buffer ) {
			let	mut buffer2= String::new();
			buffer2+= &v[1];
			buffer2+= "\x07L0";
			buffer2+= &v[2];
			buffer2+= "\x07L1";
			buffer2+= &v[3];
			buffer2+= "\x07L2";
			buffer2+= &v[4];
			buffer= buffer2;
			continue;
		}
		if let Some(v)= PAT_LINK1.captures( &buffer ) {
			let	mut buffer2= String::new();
			buffer2+= &v[1];
			buffer2+= "\x07L0";
			buffer2+= &v[2];
			buffer2+= "\x07L1";
			buffer2+= "\x07L2";
			buffer2+= &v[3];
			buffer= buffer2;
			continue;
		}
		break;
	}

	return	buffer;
}

pub fn	decode_from_doku( line0: &str ) -> String
{
	return	replace_doku_tags( line0 );
}


pub	fn	encode_to_doku( line: &str ) -> String
{
	let	mut	buffer= String::new();
	let	mut	char_it= line.chars();
	let mut slash_count= 0;
	let mut back_slash= false;
	let mut colon= false;
	loop {
		let	ch= char_it.next();
		if Some('/') == ch {
			if !colon {
				slash_count+= 1;
				continue;
			}
		}else{
			let fence= slash_count >= 2;
			if fence {
				buffer+= "<nowiki>";
			}
			for _ in 0..slash_count {
				buffer+= "/";
			}
			if fence {
				buffer+= "</nowiki>";
			}
			slash_count= 0;
		}
		if Some(':') == ch {
			colon= true;
		}else{
			colon= false;
		}
		if back_slash {
			if Some('_') != ch {
				buffer+= "\\";
			}
			back_slash= false;
		}
		match ch {
			Some('\x07') => {
				let	cmd0= char_it.next().unwrap();
				let	cmd1= char_it.next().unwrap();
				match cmd0 {
					'B'|'b' => {
						match cmd1 {
							'1' => {
								buffer+= "//";
							},
							'2' => {
								buffer+= "**";
							},
							_ => {
								buffer+= "//**";
							},
						}
					},
					'D' => {
						buffer+= "<del>";
					},
					'd' => {
						buffer+= "</del>";
					},
					'L' => {
						match cmd1 {
							'0' => {
								buffer+= "[[";
							},
							'1' => {
								buffer+= "|";
							},
							_ => {
								buffer+= "]]";
							},
						}
					},
					_ => {
					},
				}
			},
			Some('\\') => {
				back_slash= true;
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


//-----------------------------------------------------------------------------

trait ElementGenerator {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>;
}


struct GenerateorContext {
	// pre
	pre_block: bool,
	pre_code: String,
	pre_data: String,
	// table
	table_block: bool,
	table_column: Vec<TableColumn>,
	table_pat: regex::Regex,
}

impl GenerateorContext {
	fn	new() -> Self
	{
		GenerateorContext{
				pre_block: false,
				pre_code: String::new(),
				pre_data: String::new(),
				table_block: false,
				table_column: Vec::new(),
				table_pat: regex::Regex::new( r"^(\||\^)(.*)(\||\^)\s*$" ).unwrap(),
			}
	}
	//-------------------------------------------------------------------------
	fn	is_pre_block( &self ) -> bool
	{
		self.pre_block
	}
	fn	add_pre_block( &mut self, line: &str ) -> Option<Box<dyn Element>>
	{
		if line.starts_with( "</code>" ) {
			self.pre_block= false;
			return	Some( Box::new( PRETagElement{
							etype:	ElementType::PRETAG,
							text: 	(&self.pre_data).to_string(),
							code:	(&self.pre_code).to_string(),
						}));
		}
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
			let	header= &v[1] == "^";
			let	tline= &v[2];
			let	params= tline.split( &['|', '^'] );
			let	mut	column= TableColumn::new();
			for (_,td) in params.enumerate() {
				let	mut	align= ETableAlign::DEFAULT;
				let	mut	ls= 0;
				let	mut	rs= 0;
				if td.starts_with( "  " ) {
					ls= 2;
				}else if td.starts_with( " " ) {
					ls= 1;
				}
				if td.ends_with( "  " ) {
					rs= 2;
				}else if td.ends_with( " " ) {
					rs= 1;
				}
				if ls >= 2 && rs >= 2 {
					align= ETableAlign::CENTER;
				}
				if ls == 1 && rs >= 2 {
					align= ETableAlign::LEFT;
				}
				if ls >= 2 && rs == 1 {
					align= ETableAlign::RIGHT;
				}
				//println!( "[{}] {:?}", td, align );
				column.add( TableAttr{
						text: decode_from_doku( td.trim() ),
						align: align,
						header: header,
					} );
			}
			self.table_column.push( column )
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
			pat: regex::Regex::new( r"^(=+)\s+(.*)\s+(=+)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for HTagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			//println!( "{} {} {}", &v[1], &v[2], v[1].len() );
			let	hlen= v[1].len() as u32;
			let	level= if hlen < 7 { 7 - hlen }else{ 1 };
			return	Some( Box::new( HTagElement{
							etype:	ElementType::HTAG,
							title: 	v[2].to_string(),
							level:	level,
						}));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct LITagGen {
	pat: regex::Regex,
}

impl LITagGen {
	fn	new() -> Self
	{
		LITagGen{
			pat: regex::Regex::new( r"^  (\s*)(\*|\-)\s+(.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for LITagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			//println!( "ul {} {} {}", &v[2], &v[1], v[1].len() );
			let	indent= v[1].len() as u32;
			let nest= indent / 2;
			let	tag= if &v[2] == "*" { ElementType::ULTAG } else { ElementType::OLTAG };
			return	Some( Box::new( LITagElement{
							etype:	tag,
							text: 	decode_from_doku( &v[3] ),
							indent:	indent,
							nest:	nest,
						}));
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct PRETagGen {
	pat_format : regex::Regex,
	pat : regex::Regex,
}

impl PRETagGen {
	fn	new() -> Self
	{
		PRETagGen{
			pat_format: regex::Regex::new( r"^<code\s+(\w+)>$" ).unwrap(),
			pat: regex::Regex::new( r"^<code>$" ).unwrap(),
		}
	}
}

impl ElementGenerator for PRETagGen {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat_format.captures( line );
		if let Some(v)= pat_result {
			context.pre_block= true;
			context.pre_code= v[1].to_string();
			context.pre_data= String::new();
			return	Some( Box::new( NONEElement{} ));
		}
		let	pat_result= self.pat.captures( line );
		if let Some(_)= pat_result {
			context.pre_block= true;
			context.pre_code= String::new();
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
			pat: regex::Regex::new( r"^(\||\^).*(\||\^)\s*$" ).unwrap(),
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
			pat: regex::Regex::new( r"^----" ).unwrap(),
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
							page.push( Box::new(DataElement{ text: decode_from_doku( &line ) }) );
						}
					}
				},
				Err(e) => {
					return	Err(e);
				},
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
		let	count= 7 - self.level;
		let	tag= '='.to_string().repeat( count as usize );
		return	format!( "{} {} {}\n", tag, encode_to_doku( &self.title ), tag );
		//return	format!( "{} {} {}\n", tag, self.title, tag );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for LITagElement {
	fn	output( &self ) -> String
	{
		let	indent= (self.nest+1) * 2;
		let	spaces= ' '.to_string().repeat( indent as usize );
		if self.etype == ElementType::ULTAG {
			return	format!( "{}* {}\n", spaces, encode_to_doku( &self.text ) );
		}else{
			return	format!( "{}- {}\n", spaces, encode_to_doku( &self.text ) );
		}
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for PRETagElement {
	fn	output( &self ) -> String
	{
		if self.code.is_empty() {
			return	format!( "<code>\n{}</code>\n", self.text );
		}
		if self.code == "json" || self.code == "jsx" {
			return	format!( "<code javascript>\n{}</code>\n", self.text );
		}else{
			return	format!( "<code {}>\n{}</code>\n", self.code, self.text );
		}
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for TABLEElement {
	fn	output( &self ) -> String
	{
		let	mut	buffer= String::new();
		for column in &self.data {
			let	mut	bfirst= true;
			for attr in &column.data {
				if bfirst {
					bfirst= false;
					if attr.header {
						buffer+= "^";
					}else{
						buffer+= "|";
					}
				}
				let	mut	ls= " ";
				let	mut	rs= " ";
				match attr.align {
					ETableAlign::CENTER => {
						ls= "  ";
						rs= "  ";
					},
					ETableAlign::LEFT => {
						ls= " ";
						rs= "  ";
					},
					ETableAlign::RIGHT => {
						ls= "  ";
						rs= " ";
					},
					_ => {
					},
				}
				if attr.header {
					buffer+= &format!( "{}{}{}^", ls, encode_to_doku( &attr.text ), rs );
				}else{
					buffer+= &format!( "{}{}{}|", ls, encode_to_doku( &attr.text ), rs );
				}
			}
			buffer+= "\n";
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
		encode_to_doku( &self.text ) + "\n"
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




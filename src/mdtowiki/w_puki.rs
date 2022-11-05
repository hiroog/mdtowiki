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


//=============================================================================

pub fn	replace_puki_tags( line: &str ) -> String
{
	lazy_static! {
		static ref	PAT_EM_ARRAY: Vec<EMMarkPat>= vec![
				EMMarkPat{ pat: Regex::new( r"^(.*)'''([^']+)'''(.*)$" ).unwrap(),		cmd1: "\x07B1", cmd2: "\x07b1", },
				EMMarkPat{ pat: Regex::new( r"^(.*)''([^']+)''(.*)$" ).unwrap(),		cmd1: "\x07B2", cmd2: "\x07b2", },
				EMMarkPat{ pat: Regex::new( r"^(.*)%%([^']+)%%(.*)$" ).unwrap(),		cmd1: "\x07D0", cmd2: "\x07d0", },
			];
		//static ref	PAT_LINK2: Regex= Regex::new( r"^(.*)\[\[([^|]+)(>|:)(.*)\]\](.*)$" ).unwrap();
		static ref	PAT_LINK2: Regex= Regex::new( r"^(.*)\[\[([^|]+)(>)(.*)\]\](.*)$" ).unwrap();
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
			buffer2+= &v[4];
			buffer2+= "\x07L1";
			buffer2+= &v[2];
			buffer2+= "\x07L2";
			buffer2+= &v[5];
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

pub fn	decode_from_puki( line0: &str ) -> String
{
	return	replace_puki_tags( line0 );
}


pub	fn	encode_to_puki( line: &str ) -> String
{
	let	mut	buffer= String::new();
	let	mut	char_it= line.chars();
	loop {
		let	ch= char_it.next();
		match ch {
			Some('\x07') => {
				let	cmd0= char_it.next().unwrap();
				let	cmd1= char_it.next().unwrap();
				match cmd0 {
					'B'|'b' => {
						match cmd1 {
							'1' => {
								buffer+= "'''";
							},
							'2' => {
								buffer+= "''";
							},
							_ => {
								buffer+= "";
							},
						}
					},
					'D'|'d' => {
						buffer+= "%%";
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
								buffer+= "[[";
								buffer+= text;
								buffer+= ">";
								buffer+= url;
								buffer+= "]]";
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


//-----------------------------------------------------------------------------

trait ElementGenerator {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>;
}


struct GenerateorContext {
	// pre
	pre_block: bool,
	pre_code: String,
	pre_data: String,
	pre_pat: regex::Regex,
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
				pre_pat: regex::Regex::new( r"^ (.*)$" ).unwrap(),
				table_block: false,
				table_column: Vec::new(),
				table_pat: regex::Regex::new( r"^\|(.*)\|(h)?\s*$" ).unwrap(),
			}
	}
	//-------------------------------------------------------------------------
	fn	is_pre_block( &self ) -> bool
	{
		self.pre_block
	}
	fn	add_pre_block( &mut self, line: &str ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pre_pat.captures( line );
		if let Some(v)= pat_result {
			self.pre_data+= &v[1];
			self.pre_data+= "\n";
			return	None;
		}else{
			self.pre_block= false;
			return	Some( Box::new( PRETagElement{
							etype:	ElementType::PRETAG,
							text: 	(&self.pre_data).to_string(),
							code:	(&self.pre_code).to_string(),
						}));
		}
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
			println!( "table line={}", line );
			let	tline= &v[1];
			let	mut	header= false;
			if let Some(h)= v.get(2) {
				if h.as_str() == "h" {
					header= true;
				}
			}
			let	params= tline.split( '|' ).peekable();
			let	mut	column= TableColumn::new();
			for (_,td) in params.enumerate() {
				let	mut	align= ETableAlign::DEFAULT;
				let	mut	trim_td= td.trim();
				if trim_td.starts_with( "LEFT:" ) {
					align= ETableAlign::LEFT;
					trim_td= &trim_td[5..].trim();
				}else if trim_td.starts_with( "CENTER:" ) {
					align= ETableAlign::CENTER;
					trim_td= &trim_td[7..].trim();
				}else if trim_td.starts_with( "RIGHT:" ) {
					align= ETableAlign::RIGHT;
					trim_td= &trim_td[6..].trim();
				}
				column.add( TableAttr{
						text: decode_from_puki( trim_td ),
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
	pat_link : regex::Regex,
}

impl HTagGen {
	fn	new() -> Self
	{
		HTagGen{
			pat: regex::Regex::new( r"^(\*+)(.*)$" ).unwrap(),
			pat_link: regex::Regex::new( r"^(\*+)(.*)\s+\[.+\]$" ).unwrap(),
		}
	}
}

impl ElementGenerator for HTagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat_link.captures( line );
		if let Some(v)= pat_result {
			return	Some( Box::new( HTagElement{
							etype:	ElementType::HTAG,
							title: 	v[2].trim().to_string(),
							level:	v[1].len() as u32,
						}));
		}
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			return	Some( Box::new( HTagElement{
							etype:	ElementType::HTAG,
							title: 	v[2].trim().to_string(),
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
			pat_ul: regex::Regex::new( r"^(-+)(.*)$" ).unwrap(),
			pat_ol: regex::Regex::new( r"^(\++)(.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for LITagGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat_ul.captures( line );
		if let Some(v)= pat_result {
			let	indent= v[1].len() as u32;
			let nest= indent-1;
			return	Some( Box::new( LITagElement{
							etype:	ElementType::ULTAG,
							text: 	decode_from_puki( &v[2] ),
							indent:	indent,
							nest:	nest,
						}));
		}
		let	pat_result= self.pat_ol.captures( line );
		if let Some(v)= pat_result {
			let	indent= v[1].len() as u32;
			let nest= indent-1;
			return	Some( Box::new( LITagElement{
							etype:	ElementType::OLTAG,
							text: 	decode_from_puki( &v[2] ),
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
			pat: regex::Regex::new( r"^ (.*)$" ).unwrap(),
		}
	}
}

impl ElementGenerator for PRETagGen {
	fn	generate( &self, line: &str, context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(v)= pat_result {
			context.pre_block= true;
			context.pre_data= (&v[1]).to_string();
			context.pre_code= String::new();
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
			pat: regex::Regex::new( r"^\|.*\|(h)?\s*$" ).unwrap(),
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
	pat_hr : regex::Regex,
}

impl HRTagGen {
	fn	new() -> Self
	{
		HRTagGen{
			pat: regex::Regex::new( r"^----" ).unwrap(),
			pat_hr: regex::Regex::new( r"^#hr" ).unwrap(),
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
		let	pat_result= self.pat_hr.captures( line );
		if let Some(_)= pat_result {
			return	Some( Box::new( HRTagElement{} ) );
		}
		return	None;
	}
}


//-----------------------------------------------------------------------------

struct CommentGen {
	pat : regex::Regex,
}

impl CommentGen {
	fn	new() -> Self
	{
		CommentGen{
			pat: regex::Regex::new( r"^//" ).unwrap(),
		}
	}
}

impl ElementGenerator for CommentGen {
	fn	generate( &self, line: &str, _context: &mut GenerateorContext ) -> Option<Box<dyn Element>>
	{
		let	pat_result= self.pat.captures( line );
		if let Some(_)= pat_result {
			return	Some( Box::new( NONEElement{} ) );
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
			Box::new( CommentGen::new() ),
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
						}else{
							continue;
						}
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
							page.push( Box::new(DataElement{ text: decode_from_puki( &line ) }) );
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


//-----------------------------------------------------------------------------



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
		let	tag= '*'.to_string().repeat( self.level as usize );
		return	format!( "{} {}\n", tag, self.title );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for LITagElement {
	fn	output( &self ) -> String
	{
		let	indent= self.nest+1;
		if self.etype == ElementType::ULTAG {
			return	format!( "{} {}\n", '-'.to_string().repeat(indent as usize), encode_to_puki( &self.text ) );
		}else{
			return	format!( "{} {}\n", '+'.to_string().repeat(indent as usize), encode_to_puki( &self.text ) );
		}
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for PRETagElement {
	fn	output( &self ) -> String
	{
		let	mut	buffer= String::new();
		for line in self.text.split( '\n' ) {
			buffer+= " ";
			buffer+= line;
			buffer+= "\n";
		}
		return	buffer;
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
				match attr.align {
					ETableAlign::CENTER => {
						buffer+= "CENTER:";
					},
					ETableAlign::LEFT => {
						buffer+= "LEFT:";
					},
					ETableAlign::RIGHT => {
						buffer+= "RIGHT:";
					},
					_ => {
					},
				}
				buffer+= &format!( " {} |", encode_to_puki( &attr.text ) );
				if !bheader && attr.header {
					bheader= true;
				}
			}
			if bheader {
				buffer+= "h";
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
		encode_to_puki( &self.text ) + "\n"
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

impl TextEncoder for Encoder {
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




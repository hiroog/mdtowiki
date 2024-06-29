// 2022 Hiroyuki Ogasawara
// vim:ts=4 sw=4 noet:

//use std::io;

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

pub	fn	encode_to_conf( line: &str ) -> String
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
								buffer+= "_";
							},
							'2' => {
								buffer+= "*";
							},
							_ => {
								buffer+= "";
							},
						}
					},
					'D'|'d' => {
						buffer+= "~";
					},
					'C'|'c' => {
						buffer+= " ";
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
								buffer+= "|";
								buffer+= url;
								buffer+= "]";
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
		return	format!( "h{}. {}\n", self.level, self.title );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for LITagElement {
	fn	output( &self ) -> String
	{
		let	indent= self.nest+1;
		if self.etype == ElementType::ULTAG {
			return	format!( "{} {}\n", '*'.to_string().repeat(indent as usize), encode_to_conf( &self.text ) );
		}else{
			return	format!( "{} {}\n", '#'.to_string().repeat(indent as usize), encode_to_conf( &self.text ) );
		}
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for PRETagElement {
	fn	output( &self ) -> String
	{
		if self.code.is_empty() {
			return	format!( "{{code:linenumbers=false|collapse=false}}\n{}{{code}}\n", self.text );
		}
		let mut lang_type= self.code.to_string();
		if lang_type == "json" {
			lang_type= "js".to_string();
		}
		return	format!( "{{code:language={}|linenumbers=false|collapse=false}}\n{}{{code}}\n", lang_type, self.text );
	}
}


//-----------------------------------------------------------------------------

impl	EncodeElement for TABLEElement {
	fn	output( &self ) -> String
	{
		let	mut	buffer= String::new();
		for column in &self.data {
			let mut cindex= 0;
			for attr in &column.data {
				if attr.header {
					if cindex == 0 {
						buffer+= "||";
					}
					buffer+= &format!( " {} ||", encode_to_conf( &attr.text ) );
				}else{
					if cindex == 0 {
						buffer+= "|";
					}
					buffer+= &format!( " {} |", encode_to_conf( &attr.text ) );
				}
				cindex+= 1;
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
		encode_to_conf( &self.text ) + "\n"
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




// 2022 Hiroyuki Ogasawara
// vim:ts=4 sw=4 noet:

use std::fs;
use std::io::{self,Write};
use super::element::*;

//-----------------------------------------------------------------------------

pub struct Document {
	pub element_list	: Vec<Box<dyn Element>>,
}

pub trait TextEncoder {
	fn	encode_single( &self, element: &Box<dyn Element> ) -> String;
	fn	encode( &self, document: &Document ) -> String
	{
		let	mut	buffer= String::new();
		for element in &document.element_list {
			buffer+= &self.encode_single( element );
		}
		return	buffer;
	}
	fn	save( &self, file_name: &str, document: &Document ) -> io::Result<()>
	{
		let	mut fp= fs::File::create( file_name )?;
		fp.write_all( &self.encode( document ).as_bytes() )?;
		fp.flush()?;
		Ok(())
	}
}

impl Document {
	pub fn	new() -> Self
	{
		Document{ element_list: Vec::new() }
	}
	pub fn	push( &mut self, element: Box<dyn Element> )
	{
		self.element_list.push( element );
	}

	pub	fn	dump( &self )
	{
		for element in &self.element_list {
			element.dump();
		}
	}
/*
	pub	fn	save_file( &self, file_name: &str, data: &str ) -> io::Result<()>
	{
		let	mut fp= fs::File::create( file_name )?;
		fp.write_all( &data.as_bytes() )?;
		fp.flush()?;
		Ok(())
	}
*/
	pub	fn	save<T:TextEncoder>( &self, file_name: &str, encoder: &T ) -> io::Result<()>
	{
		let	mut fp= fs::File::create( file_name )?;
		fp.write_all( &encoder.encode( &self ).as_bytes() )?;
		fp.flush()?;
		Ok(())
	}
}


//-----------------------------------------------------------------------------




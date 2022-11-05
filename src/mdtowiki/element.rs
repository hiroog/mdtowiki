// 2022 Hiroyuki Ogasawara
// vim:ts=4 sw=4 noet:

use std::any::Any;

//-----------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum ElementType {
	NONE,
	DATA,
	HTAG,	// <h#>
	ULTAG,	// <ul><li>
	OLTAG,	// <ol><li>
	PRETAG,	// <pre> or <code>
	TABLE,	// <table><tr><td>
	HRTAG,	// <hr/>
}

//-----------------------------------------------------------------------------

pub trait CastToAny : Any {
	fn	as_any( &self ) -> &dyn Any;
}

pub trait Element : CastToAny {
	fn	get_type( &self ) -> ElementType;
	fn	dump( &self );
}

impl<T:Element>	CastToAny for T {
	fn	as_any( &self ) -> &dyn Any
	{
		self
	}
}

//=============================================================================

pub struct NONEElement {
}

impl Element for NONEElement {
	fn	get_type( &self ) -> ElementType
	{
		return	ElementType::NONE;
	}
	fn	dump( &self )
	{
	}
}

//-----------------------------------------------------------------------------

pub struct HTagElement {
	pub title	: String,
	pub etype	: ElementType,
	pub level	: u32,	// 1, 2, 3,
}

impl Element for HTagElement {
	fn	get_type( &self ) -> ElementType
	{
		return	self.etype;
	}
	fn	dump( &self )
	{
		println!( "h{} {}", self.level, self.title );
	}
}

//-----------------------------------------------------------------------------

pub struct LITagElement {
	pub text	: String,
	pub etype	: ElementType,
	pub indent	: u32,
	pub nest	: u32,	// 0, 1, 2,
}

impl Element for LITagElement {
	fn	get_type( &self ) -> ElementType
	{
		return	self.etype;
	}
	fn	dump( &self )
	{
		if self.etype == ElementType::ULTAG {
			println!( "ul {} ({}) {}", self.nest, self.indent, self.text );
		}else if self.etype == ElementType::OLTAG {
			println!( "ol {} ({}) {}", self.nest, self.indent, self.text );
		}
	}
}


//-----------------------------------------------------------------------------

pub struct PRETagElement {
	pub text	: String,
	pub etype	: ElementType,
	pub code	: String,
}

impl Element for PRETagElement {
	fn	get_type( &self ) -> ElementType
	{
		return	self.etype;
	}
	fn	dump( &self )
	{
		println!( "pre {} {}", self.code, self.text );
	}
}


//-----------------------------------------------------------------------------

#[derive(Clone,Copy,Debug)]
pub enum ETableAlign {
	DEFAULT,
	CENTER,
	LEFT,
	RIGHT,
}

#[derive(Clone)]
pub struct TableAttr {
	pub text	: String,
	pub align   : ETableAlign,
	pub header  : bool,
}

#[derive(Clone)]
pub struct TableColumn {
	pub data	: Vec<TableAttr>,
}

impl TableColumn {
	pub fn	new() -> Self
	{
		TableColumn{ data: Vec::new() }
	}
	pub	fn	add( &mut self, attr: TableAttr )
	{
		self.data.push( attr );
	}
}

pub struct TABLEElement {
	pub etype	: ElementType,
	pub data	: Vec<TableColumn>,
}

impl Element for TABLEElement {
	fn	get_type( &self ) -> ElementType
	{
		return	self.etype;
	}
	fn	dump( &self )
	{
		println!( "table" );
		for column in &self.data {
			for attr in &column.data {
				if attr.header {
					print!( "^ {} ^", attr.text );
				}else{
					print!( "| {} |", attr.text );
				}
			}
			println!( "" );
		}
	}
}


//-----------------------------------------------------------------------------

pub struct HRTagElement {
}

impl Element for HRTagElement {
	fn	get_type( &self ) -> ElementType
	{
		return	ElementType::HRTAG;
	}
	fn	dump( &self )
	{
		println!( "hr" );
	}
}


//-----------------------------------------------------------------------------

pub struct DataElement {
	pub text	: String,
}

impl Element for DataElement {
	fn	get_type( &self ) -> ElementType
	{
		return	ElementType::DATA;
	}
	fn	dump( &self )
	{
		println!( "data {}", self.text );
	}
}

//-----------------------------------------------------------------------------




use scale_info::prelude::{boxed::Box, string::String};

pub fn convert_error_message(error_message: String) -> &'static str {
	Box::leak(error_message.into_boxed_str())
}

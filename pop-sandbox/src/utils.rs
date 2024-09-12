use std::error::Error;

use drink::{
	session::{Session, NO_ARGS},
	AccountId32,
};

use super::*;

// TODO: Convert this method to accept generic runtime instead of `PopSandbox`.
pub fn call_function(
	mut sess: Session<PopSandbox>,
	contract: &AccountId32,
	sender: &AccountId32,
	func_name: String,
	args: Option<Vec<String>>,
	value: Option<u128>,
) -> Result<Session<PopSandbox>, Box<dyn Error>> {
	sess.set_actor(sender.clone());
	if let Some(args) = args {
		println!("Calling function: {}() | Input: {:?}", func_name, args);
		sess.call_with_address(contract.clone(), &func_name, &args, value)??;
	} else {
		println!("Calling function: {}() | Input: None", func_name);
		sess.call_with_address(contract.clone(), &func_name, NO_ARGS, value)??;
	}

	let encoded = &sess.record().last_call_result().result.clone().unwrap().data;
	let decoded = encoded.iter().map(|b| *b as char).collect::<String>();
	let messages: Vec<String> = decoded.split('\n').map(|s| s.to_string()).collect();
	// Print debug logs
	for line in messages {
		if line.len() > 0 {
			println!("LOG: {}", line);
		}
	}

	Ok(sess)
}


use ink::prelude::string::String;
use pop_api::{
	primitives::TokenId ,
	v0::fungibles::{
		self as api,
	},
};

pub trait Api{
    fn token_name(&self, id: TokenId) -> Option<String>;
    fn token_symbol(id: TokenId) -> Option<String>;
    fn token_decimals(&self, id: TokenId) -> u8;
    /*fn mint(&self, id: TokenId, account: AccountId, value: u128) -> Result<(), Psp22Error>;
    fn burn(&self, id: TokenId, account: AccountId, value: u128) -> Result<(), Psp22Error>;*/
}

pub struct ApiImpl;

impl Api for ApiImpl{
    
    fn token_name(&self, id: TokenId) -> Option<String> {
        api::token_name(id)
				.unwrap_or_default()
				.and_then(|v| String::from_utf8(v).ok())
    }

    
    fn token_symbol(id: TokenId) -> Option<String> {
        api::token_symbol(id)
            .unwrap_or_default()
            .and_then(|v| String::from_utf8(v).ok())
    }

    
    fn token_decimals(&self, id: TokenId) -> u8 {
        api::token_decimals(id).unwrap_or_default()
    }

    /*
    fn mint(&self, id: TokenId, account: AccountId, value: u128) -> Result<(), Psp22Error> {
        self.ensure_owner()?;
        // No-op if `value` is zero.
        if value == 0 {
            return Ok(());
        }
        api::mint(id, account, value).map_err(Psp22Error::from)?;
        self.env().emit_event(Transfer { from: None, to: Some(account), value });
        Ok(())
    }

   
    fn burn(&self, id: TokenId, account: AccountId, value: u128) -> Result<(), Psp22Error>{
        self.ensure_owner()?;
        // No-op if `value` is zero.
        if value == 0 {
            return Ok(());
        }
        api::burn(id, account, value).map_err(Psp22Error::from)?;
        self.env().emit_event(Transfer { from: Some(account), to: None, value });
        Ok(())
    }*/
    
}
#![no_main]
#![no_std]

use ckb_cinnabar_verifier::{
    cinnabar_main, define_errors, Result, Verification, CUSTOM_ERROR_START, TREE_ROOT,
};
use ckb_std::{
    ckb_constants::Source::Input,
    debug,
    high_level::{load_cell_type, QueryIter},
};

mod hardcoded;

define_errors!(
    ScriptError,
    {
        NoDaoCertificateFound = CUSTOM_ERROR_START,
    }
);

#[derive(Default)]
struct Context {}

// Check if dao-certificate script appears in Inputs of transaction
#[derive(Default)]
struct Root {}

impl Verification<Context> for Root {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("verifying {}", name);

        let find = QueryIter::new(load_cell_type, Input).any(|script| {
            if let Some(script) = script {
                script.code_hash().raw_data().as_ref() == hardcoded::DAO_CERTIFICATE_TYPE_HASH
            } else {
                false
            }
        });
        if !find {
            return Err(ScriptError::NoDaoCertificateFound.into());
        }

        Ok(None)
    }
}

cinnabar_main!(Context, (TREE_ROOT, Root));

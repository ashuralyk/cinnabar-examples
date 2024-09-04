#![no_main]
#![no_std]

use blake2b_ref::Blake2bBuilder;
use ckb_cinnabar_verifier::{
    cinnabar_main, define_errors, Result, Verification, CUSTOM_ERROR_START, TREE_ROOT,
};
use ckb_std::{
    ckb_constants::Source::{GroupInput, GroupOutput, Input, Output},
    ckb_types::packed::Script,
    debug,
    high_level::{
        load_cell, load_cell_capacity, load_cell_data, load_cell_lock, load_cell_type,
        load_cell_type_hash, load_header, load_input, load_script, QueryIter,
    },
};
use generated::SporeData;
use molecule::prelude::Entity;

mod generated;
mod hardcoded;

define_errors!(
    ScriptError,
    {
        UnknownPattern = CUSTOM_ERROR_START,
        UnexpectedTypeId,
        DaoCellNotFound,
        InvalidCertificateDataFormat,
        DaoCapacityNotMatch,
        UnsupportedDoubleMint,
        DaoCellNotLocked,
        SporeCellNotFound,
        SporeCellNotLocked,
        InvalidSporeData,
        UnexpectedSporeDataFormat,
    }
);

#[derive(Default)]
struct Context {}

// Analyze the operation type of this transaction, and then dispatch into different verification logic
#[derive(Default)]
struct Root {}

impl Verification<Context> for Root {
    fn verify(&mut self, name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("verifying {}", name);

        let in_input = load_cell(0, GroupInput).is_ok();
        let in_output = load_cell(0, GroupOutput).is_ok();

        match (in_input, in_output) {
            (false, true) => Ok(Some("deposit")),
            (true, false) => Ok(Some("withdraw")),
            (true, true) => Ok(Some("mint")),
            _ => Err(ScriptError::UnknownPattern.into()),
        }
    }
}

// Verify the case of DAO deposit with dao-certificate cell created at the same time
#[derive(Default)]
struct DaoDeposit {}

impl DaoDeposit {
    fn calc_type_id(expected_script: &Script) -> Result<[u8; 32]> {
        let first_input = load_input(0, Input)?;
        let output_index = QueryIter::new(load_cell_type, Output)
            .enumerate()
            .find_map(|(i, script)| {
                if script.as_ref() == Some(expected_script) {
                    Some(i)
                } else {
                    None
                }
            })
            .unwrap();
        let mut blake2b = Blake2bBuilder::new(32)
            .personal(b"ckb-default-hash")
            .build();
        blake2b.update(first_input.as_slice());
        blake2b.update(&(output_index as u64).to_le_bytes());
        let mut type_id = [0; 32];
        blake2b.finalize(&mut type_id);
        Ok(type_id)
    }
}

impl Verification<Context> for DaoDeposit {
    fn verify(&mut self, verifier_name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("verifying {}", verifier_name);

        // Check the type-id format
        let script = load_script()?;
        let expected_type_id = Self::calc_type_id(&script)?;
        if script.args().raw_data().as_ref() != expected_type_id {
            return Err(ScriptError::UnexpectedTypeId.into());
        }

        // Find DAO cell information
        let Some((dao_index, expected_dao_capacity)) = QueryIter::new(load_cell_type, Output)
            .enumerate()
            .find_map(|(i, type_script)| {
                if let Some(script) = type_script {
                    if script.code_hash().raw_data().as_ref() == hardcoded::DAO_TYPE_HASH {
                        let capacity = load_cell_capacity(i, Output).unwrap();
                        Some((i, capacity))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        else {
            return Err(ScriptError::DaoCellNotFound.into());
        };

        // The dao-certificate cell must record deposit capacity of DAO in its cell data
        let data = load_cell_data(0, GroupOutput)?;
        let dao_capacity = u64::from_le_bytes(
            data[..8]
                .try_into()
                .map_err(|_| ScriptError::InvalidCertificateDataFormat)?,
        );

        // Check if the deposit capacity of DAO is correct
        if dao_capacity != expected_dao_capacity {
            return Err(ScriptError::DaoCapacityNotMatch.into());
        }

        // Check if the lock script of DAO cell is locked by type-burn script
        let type_burn_lock_script = load_cell_lock(dao_index, Output)?;
        let code_hash = type_burn_lock_script.code_hash().raw_data();
        let args = type_burn_lock_script.args().raw_data();
        if code_hash.as_ref() != hardcoded::TYPE_BURN_CODE_HASH
            && args.as_ref() != load_cell_type_hash(0, GroupOutput)?.unwrap()
        {
            return Err(ScriptError::DaoCellNotLocked.into());
        }

        Ok(None)
    }
}

// Verify the case of minting Spore DOB cell after the DAO deposit
#[derive(Default)]
struct SporeMint {}

impl Verification<Context> for SporeMint {
    fn verify(&mut self, verifier_name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("verifying {}", verifier_name);

        // Find spore dob cell information
        let Some((spore_data, spore_type_hash)) = QueryIter::new(load_cell_type, Output)
            .enumerate()
            .find_map(|(i, type_script)| {
                if let Some(script) = type_script {
                    if script.code_hash().raw_data().as_ref() == hardcoded::SPORE_CODE_HASH {
                        let data = load_cell_data(i, Output).unwrap();
                        let type_hash = script.calc_script_hash().raw_data().to_vec();
                        Some((data, type_hash))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        else {
            return Err(ScriptError::SporeCellNotFound.into());
        };

        // Check if the lock script of dao-certificate cell in input is not a type-burn script
        let lock_script = load_cell_lock(0, GroupInput)?;
        if lock_script.code_hash().raw_data().as_ref() == hardcoded::TYPE_BURN_CODE_HASH {
            return Err(ScriptError::UnsupportedDoubleMint.into());
        }

        // Check if the lock script of dao-certificate cell in output is locked by spore dob
        let type_burn_lock_script = load_cell_lock(0, GroupOutput)?;
        let code_hash = type_burn_lock_script.code_hash().raw_data();
        let args = type_burn_lock_script.args().raw_data();
        if code_hash.as_ref() != hardcoded::TYPE_BURN_CODE_HASH
            && args.as_ref() != spore_type_hash.as_slice()
        {
            return Err(ScriptError::SporeCellNotLocked.into());
        }

        // Check if parts of the data of spore dob cell are correct
        let spore_data = SporeData::from_compatible_slice(&spore_data)
            .map_err(|_| ScriptError::InvalidSporeData)?;
        let content_type = spore_data.content_type().raw_data();
        let cluster_id = spore_data.cluster_id().to_opt();
        if content_type.as_ref() != b"dob/1" || cluster_id.is_none() {
            return Err(ScriptError::UnexpectedSporeDataFormat.into());
        }

        // Check if particularly the dob content is correct
        let content = spore_data.content().raw_data();
        let dao_capacity = load_cell_data(0, GroupOutput)?;
        let dao_certificate_header = load_header(0, GroupOutput)?;
        let expected_content = {
            let mut content = dao_capacity;
            content.append(&mut dao_certificate_header.raw().number().raw_data().to_vec());
            content
        };
        if content.as_ref() != &expected_content {
            return Err(ScriptError::UnexpectedSporeDataFormat.into());
        }

        Ok(None)
    }
}

// Empty verification logic for DAO withdraw
//
// note: no need to verify the withdraw operation, because the DAO cell is locked by type-burn script,
//       if the wrong transation is provided, DAO cell will be locked forever and its inner capacity will be burned
#[derive(Default)]
struct DaoWithdraw {}

impl Verification<Context> for DaoWithdraw {
    fn verify(&mut self, verifier_name: &str, _: &mut Context) -> Result<Option<&str>> {
        debug!("verifying {}", verifier_name);

        Ok(None)
    }
}

cinnabar_main!(
    Context,
    (TREE_ROOT, Root),
    ("deposit", DaoDeposit)("mint", SporeMint)("withdraw", DaoWithdraw)
);

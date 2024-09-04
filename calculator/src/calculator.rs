use ckb_cinnabar::calculator::{
    instruction::Instruction,
    operation::{
        basic::AddOutputCellByInputIndex,
        component::{
            AddLockProxyInputCell, AddLockProxyOutputCell, AddTypeBurnInputCellByInputIndex,
        },
        spore::AddSporeInputCellBySporeId,
    },
    re_exports::{ckb_sdk::Address, ckb_types::H256},
    rpc::RPC,
    skeleton::ScriptEx,
};

use crate::{
    config::DAO_CERTIFICATE_CHECK_NAME,
    operation::{
        AddDaoCertificateCheckCelldep, AddDaoCertificateInputCell,
        AddDaoCertificateOutputCellWithDaoDeposit, AddDaoCertificateOutputCellWithSporeTypeBurn,
    },
};

pub fn dao_deposit_with_certificate<T: RPC>(
    depositer: Address,
    dao_capacity: u64,
) -> Instruction<T> {
    Instruction::new(vec![Box::new(AddDaoCertificateOutputCellWithDaoDeposit {
        depositer: depositer.into(),
        dao_capacity,
    })])
}

pub fn create_depositer_lock_proxy_cell<T: RPC>(depositer: Address) -> Instruction<T> {
    Instruction::new(vec![
        Box::new(AddDaoCertificateCheckCelldep {}),
        Box::new(AddLockProxyOutputCell {
            lock_hash: ScriptEx::from(depositer.clone()).script_hash().unwrap(),
            lock_script: false,
            second_script: Some((DAO_CERTIFICATE_CHECK_NAME.to_owned(), vec![]).into()),
            data: vec![],
        }),
    ])
}

pub fn spore_mint_with_certificate<T: RPC>(depositer: Address, cluster_id: H256) -> Instruction<T> {
    Instruction::new(vec![
        Box::new(AddLockProxyInputCell {
            lock_hash: ScriptEx::from(depositer.clone()).script_hash().unwrap(),
            lock_script: false,
            count: 1,
        }),
        Box::new(AddOutputCellByInputIndex {
            input_index: usize::MAX,
            data: None,
            lock_script: None,
            type_script: None,
            adjust_capacity: false,
        }),
        Box::new(AddDaoCertificateInputCell {
            depositer: depositer.into(),
        }),
        Box::new(AddDaoCertificateOutputCellWithSporeTypeBurn {
            certificate_index: usize::MAX,
            cluster_id,
        }),
    ])
}

pub fn dao_withdraw_with_certificate<T: RPC>(depositer: Address, spore_id: H256) -> Instruction<T> {
    Instruction::new(vec![
        Box::new(AddSporeInputCellBySporeId {
            spore_id,
            check_owner: Some(depositer.into()),
        }),
        // dao certificate cell
        Box::new(AddTypeBurnInputCellByInputIndex {
            input_index: usize::MAX,
        }),
        // dao deposit cell
        Box::new(AddTypeBurnInputCellByInputIndex {
            input_index: usize::MAX,
        }),
    ])
}

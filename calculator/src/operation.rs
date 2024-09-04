use ckb_cinnabar::calculator::{
    operation::{
        basic::{AddCellDep, AddInputCell, AddOutputCell},
        component::{
            hardcoded::{Name, TYPE_BURN_CODE_HASH},
            AddComponentCelldep, AddTypeBurnOutputCell,
        },
        dao::AddDaoDepositOutputCell,
        spore::{AddSporeOutputCell, ClusterAuthorityMode},
        Log, Operation,
    },
    re_exports::{
        async_trait,
        ckb_sdk::rpc::ckb_indexer::SearchMode,
        ckb_types::{core::DepType, prelude::Unpack, H256},
        eyre,
    },
    rpc::{Network, RPC},
    simulation::{AddFakeContractCelldepByName, AddFakeInputCell},
    skeleton::{HeaderDepEx, ScriptEx, TransactionSkeleton},
};

use crate::config;

/// Add dao-certificate-type Celldep
pub struct AddDaoCertificateCelldep {}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateCelldep {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        match rpc.network() {
            Network::Mainnet | Network::Testnet => {
                let deployment = config::dao_certificate_deployment(rpc.network())?;
                Box::new(AddCellDep {
                    name: config::DAO_CERTIFICATE_NAME.to_string(),
                    tx_hash: deployment.tx_hash.clone(),
                    index: deployment.out_index,
                    with_data: false,
                    dep_type: DepType::Code,
                })
                .run(rpc, skeleton, log)
                .await
            }
            _ => {
                Box::new(AddFakeContractCelldepByName {
                    contract: config::DAO_CERTIFICATE_NAME.to_string(),
                    with_type_id: true,
                    contract_binary_path: "../build/release".to_string(),
                })
                .run(rpc, skeleton, log)
                .await
            }
        }
    }
}

/// Add dao-certificate-check-lock Celldep
pub struct AddDaoCertificateCheckCelldep {}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateCheckCelldep {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        match rpc.network() {
            Network::Mainnet | Network::Testnet => {
                let deployment = config::dao_certificate_check_deployment(rpc.network())?;
                Box::new(AddCellDep {
                    name: config::DAO_CERTIFICATE_CHECK_NAME.to_string(),
                    tx_hash: deployment.tx_hash.clone(),
                    index: deployment.out_index,
                    with_data: false,
                    dep_type: DepType::Code,
                })
                .run(rpc, skeleton, log)
                .await
            }
            _ => {
                Box::new(AddFakeContractCelldepByName {
                    contract: config::DAO_CERTIFICATE_CHECK_NAME.to_string(),
                    with_type_id: true,
                    contract_binary_path: "../build/release".to_string(),
                })
                .run(rpc, skeleton, log)
                .await
            }
        }
    }
}

/// Add both dao-certificate cell and dao-deposit cell
///
/// note: the dao-deposit cell points to dao-certificate cell by type-burn
///
/// # Parameters
/// - `depositer`: the lock script of the dao depositer
/// - `dao_capacity`: the deposit capacity
pub struct AddDaoCertificateOutputCellWithDaoDeposit {
    pub depositer: ScriptEx,
    pub dao_capacity: u64,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateOutputCellWithDaoDeposit {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        Box::new(AddDaoCertificateCelldep {})
            .run(rpc, skeleton, log)
            .await?;
        let self_type_id = skeleton.calc_type_id(skeleton.outputs.len())?;
        let dao_certificate_script = ScriptEx::from((
            config::DAO_CERTIFICATE_NAME.to_string(),
            self_type_id.as_bytes().to_vec(),
        ))
        .to_script(skeleton)?;
        let type_hash: H256 = dao_certificate_script.calc_script_hash().unpack();
        Box::new(AddOutputCell {
            lock_script: self.depositer,
            type_script: Some(dao_certificate_script.into()),
            data: self.dao_capacity.to_le_bytes().to_vec(),
            capacity: 0,
            absolute_capacity: false,
            type_id: false,
        })
        .run(rpc, skeleton, log)
        .await?;
        let type_burn_lock_script =
            ScriptEx::new_code(TYPE_BURN_CODE_HASH, type_hash.as_bytes().to_vec());
        Box::new(AddDaoDepositOutputCell {
            owner: type_burn_lock_script,
            deposit_capacity: self.dao_capacity,
        })
        .run(rpc, skeleton, log)
        .await
    }
}

pub struct AddDaoCertificateInputCell {
    pub depositer: ScriptEx,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateInputCell {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        Box::new(AddDaoCertificateCelldep {})
            .run(rpc, skeleton, log)
            .await?;
        let partial_type_script =
            ScriptEx::from((config::DAO_CERTIFICATE_NAME.to_string(), vec![]));
        Box::new(AddInputCell {
            lock_script: self.depositer,
            type_script: Some(partial_type_script),
            count: 1,
            search_mode: SearchMode::Prefix,
        })
        .run(rpc, skeleton, log)
        .await
    }
}

pub struct AddDaoCertificateTypeBurnInputCell {
    pub input_index: usize,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateTypeBurnInputCell {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        Box::new(AddDaoCertificateCelldep {})
            .run(rpc, skeleton, log)
            .await?;
        Box::new(AddComponentCelldep {
            name: Name::TypeBurn,
        })
        .run(rpc, skeleton, log)
        .await?;
        let dao_certificate_cell = skeleton.get_input_by_index(self.input_index)?;
        let type_hash = dao_certificate_cell
            .output
            .calc_type_hash()
            .ok_or(eyre::eyre!("dao certificate cell should have type script"))?;
        let type_burn_lock =
            ScriptEx::new_code(TYPE_BURN_CODE_HASH, type_hash.as_bytes().to_owned());
        if rpc.network() != Network::Fake {
            let partial_type_script =
                ScriptEx::from((config::DAO_CERTIFICATE_NAME.to_string(), vec![]));
            Box::new(AddInputCell {
                lock_script: type_burn_lock,
                type_script: Some(partial_type_script),
                count: 1,
                search_mode: SearchMode::Prefix,
            })
            .run(rpc, skeleton, log)
            .await
        } else {
            let dao_certificate_type =
                ScriptEx::from((config::DAO_CERTIFICATE_NAME.to_string(), vec![1u8; 32]))
                    .to_script(skeleton)?;
            let dao_capacity = 0u64.to_le_bytes().to_vec();
            Box::new(AddFakeInputCell {
                lock_script: type_burn_lock.into(),
                type_script: Some(dao_certificate_type.into()),
                data: dao_capacity,
                capacity: 0,
                absolute_capacity: false,
            })
            .run(rpc, skeleton, log)
            .await
        }
    }
}

pub struct AddDaoCertificateOutputCellWithSporeTypeBurn {
    pub certificate_index: usize,
    pub cluster_id: H256,
}

#[async_trait::async_trait]
impl<T: RPC> Operation<T> for AddDaoCertificateOutputCellWithSporeTypeBurn {
    async fn run(
        self: Box<Self>,
        rpc: &T,
        skeleton: &mut TransactionSkeleton,
        log: &mut Log,
    ) -> eyre::Result<()> {
        let dao_certificate_cell = skeleton.get_input_by_index(self.certificate_index)?;
        let depositer = dao_certificate_cell.output.lock_script();
        let dao_deposit_header =
            HeaderDepEx::new_from_outpoint(rpc, dao_certificate_cell.input.previous_output())
                .await?;
        let dao_capacity = u64::from_le_bytes(
            dao_certificate_cell.output.data[..8]
                .try_into()
                .map_err(|_| eyre::eyre!("invalid dao certificate data"))?,
        );
        let dao_certificate_type = dao_certificate_cell
            .output
            .type_script()
            .ok_or_else(|| eyre::eyre!("dao certificate cell should have type script"))?;
        let content = vec![
            dao_capacity.to_le_bytes().to_vec(),
            dao_deposit_header.header.number().to_le_bytes().to_vec(),
        ]
        .concat();
        Box::new(AddSporeOutputCell {
            lock_script: depositer.into(),
            content_type: "dob/1".to_owned(),
            content,
            cluster_id: Some(self.cluster_id),
            authority_mode: ClusterAuthorityMode::Skip,
        })
        .run(rpc, skeleton, log)
        .await?;
        Box::new(AddTypeBurnOutputCell {
            output_index: usize::MAX,
            type_script: Some(dao_certificate_type.into()),
            data: dao_capacity.to_le_bytes().to_vec(),
        })
        .run(rpc, skeleton, log)
        .await?;
        skeleton.headerdep(dao_deposit_header);
        Ok(())
    }
}

use ckb_cinnabar::{
    calculator::{
        re_exports::eyre::{eyre, Result},
        rpc::Network,
        simulation::random_hash,
    },
    load_contract_deployment, DeploymentRecord,
};
use lazy_static::lazy_static;

pub const DAO_CERTIFICATE_NAME: &str = "dao-certificate-type";
pub const DAO_CERTIFICATE_CHECK_NAME: &str = "dao-certificate-check-lock";

lazy_static! {
    pub static ref MAINNET_DAO_CERTIFICATE_DEPLOYMENT: Option<DeploymentRecord> =
        load_contract_deployment(
            &Network::Mainnet,
            DAO_CERTIFICATE_NAME,
            "../deployment",
            None
        )
        .expect("dao-certificate deployment not found");
    pub static ref TESTNET_DAO_CERTIFICATE_DEPLOYMENT: Option<DeploymentRecord> =
        load_contract_deployment(
            &Network::Testnet,
            DAO_CERTIFICATE_NAME,
            "../deployment",
            None
        )
        .expect("dao-certificate deployment not found");
    pub static ref MAINNET_DAO_CERTIFICATE_CHECK_DEPLOYMENT: Option<DeploymentRecord> =
        load_contract_deployment(
            &Network::Mainnet,
            DAO_CERTIFICATE_CHECK_NAME,
            "../deployment",
            None
        )
        .expect("dao-certificate-check deployment not found");
    pub static ref TESTNET_DAO_CERTIFICATE_CHECK_DEPLOYMENT: Option<DeploymentRecord> =
        load_contract_deployment(
            &Network::Testnet,
            DAO_CERTIFICATE_CHECK_NAME,
            "../deployment",
            None
        )
        .expect("dao-certificate-check deployment not found");
}

lazy_static! {
    pub static ref FAKENET_DAO_CERTIFICATE_DEPLOYMENT: DeploymentRecord = DeploymentRecord {
        name: DAO_CERTIFICATE_NAME.to_string(),
        tx_hash: random_hash().into(),
        out_index: 0,
        ..Default::default()
    };
    pub static ref FAKENET_DAO_CERTIFICATE_CHECK_DEPLOYMENT: DeploymentRecord = DeploymentRecord {
        name: DAO_CERTIFICATE_CHECK_NAME.to_string(),
        tx_hash: random_hash().into(),
        out_index: 0,
        ..Default::default()
    };
}

pub fn dao_certificate_deployment(network: Network) -> Result<&'static DeploymentRecord> {
    match network {
        Network::Mainnet => MAINNET_DAO_CERTIFICATE_DEPLOYMENT.as_ref(),
        Network::Testnet => TESTNET_DAO_CERTIFICATE_DEPLOYMENT.as_ref(),
        _ => Some(&*FAKENET_DAO_CERTIFICATE_DEPLOYMENT),
    }
    .ok_or(eyre!("dao-certificate deployment not found"))
}

pub fn dao_certificate_check_deployment(network: Network) -> Result<&'static DeploymentRecord> {
    match network {
        Network::Mainnet => MAINNET_DAO_CERTIFICATE_CHECK_DEPLOYMENT.as_ref(),
        Network::Testnet => TESTNET_DAO_CERTIFICATE_CHECK_DEPLOYMENT.as_ref(),
        _ => Some(&*FAKENET_DAO_CERTIFICATE_CHECK_DEPLOYMENT),
    }
    .ok_or(eyre!("dao-certificate-check deployment not found"))
}

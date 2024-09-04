use ckb_cinnabar::calculator::{
    instruction::Instruction,
    operation::component::{hardcoded::Name, AddComponentCelldep},
    re_exports::{
        ckb_sdk::{Address, NetworkType},
        eyre, tokio,
    },
    simulation::{
        always_success_script, fake_outpoint, random_hash, AddAlwaysSuccessCelldep,
        AddFakeContractCelldepByName, FakeRpcClient, TransactionSimulator, DEFUALT_MAX_CYCLES,
    },
    skeleton::{CellOutputEx, ScriptEx},
    TransactionCalculator,
};
use dao_certificate_calculator::{
    calculator::spore_mint_with_certificate,
    config::{DAO_CERTIFICATE_CHECK_NAME, DAO_CERTIFICATE_NAME},
    operation::{AddDaoCertificateCelldep, AddDaoCertificateCheckCelldep},
};

#[tokio::test]
async fn test_spore_mint_with_certificate() -> eyre::Result<()> {
    let mut rpc = FakeRpcClient::default();
    let prepare_celldep = Instruction::<FakeRpcClient>::new(vec![
        Box::new(AddAlwaysSuccessCelldep {}),
        Box::new(AddDaoCertificateCelldep {}),
        Box::new(AddDaoCertificateCheckCelldep {}),
        Box::new(AddFakeContractCelldepByName {
            contract: "cluster".to_string(),
            with_type_id: false,
            contract_binary_path: "./binaries".to_string(),
        }),
        Box::new(AddComponentCelldep {
            name: Name::LockProxy,
        }),
    ]);
    let (skeleton, _) = TransactionCalculator::default()
        .instruction(prepare_celldep)
        .new_skeleton(&rpc)
        .await?;

    // prepare fake lock-proxy input cell
    let depositer = always_success_script(vec![0]);
    let dao_certificate_check_script =
        ScriptEx::from((DAO_CERTIFICATE_CHECK_NAME.to_owned(), vec![]));
    let lock_proxy_script = ScriptEx::from((
        Name::LockProxy.to_string(),
        depositer.calc_script_hash().raw_data().to_vec(),
    ));
    let fake_lock_proxy_input_cell = CellOutputEx::new_from_scripts(
        dao_certificate_check_script.clone().to_script(&skeleton)?,
        Some(lock_proxy_script.to_script(&skeleton)?),
        vec![],
        None,
    )?;
    rpc.insert_fake_cell(fake_outpoint(), fake_lock_proxy_input_cell);

    // prepare dao-certificate input cell
    let dao_certificate_type =
        ScriptEx::from((DAO_CERTIFICATE_NAME.to_string(), random_hash().to_vec()));
    let dao_capacity = 1000u64.to_le_bytes().to_vec();
    let fake_dao_certificate_input_cell = CellOutputEx::new_from_scripts(
        depositer.clone().into(),
        Some(dao_certificate_type.to_script(&skeleton)?),
        dao_capacity,
        None,
    )?;
    rpc.insert_fake_cell(fake_outpoint(), fake_dao_certificate_input_cell);

    // prepare cluster celldep cell
    let cluster_id = random_hash();
    let cluster_script = ScriptEx::from(("cluster".to_string(), cluster_id.to_vec()));
    let fake_cluster_celldep_cell = CellOutputEx::new_from_scripts(
        dao_certificate_check_script.to_script(&skeleton)?,
        Some(cluster_script.to_script(&skeleton)?),
        vec![],
        None,
    )?;
    rpc.insert_fake_cell(fake_outpoint(), fake_cluster_celldep_cell);

    // run
    let depositer = Address::new(NetworkType::Dev, depositer.into(), true);
    let mint = spore_mint_with_certificate::<FakeRpcClient>(depositer, cluster_id.into());
    let cycle = TransactionSimulator::default().verify(
        &rpc,
        vec![mint],
        DEFUALT_MAX_CYCLES,
        Some(skeleton),
    )?;
    println!("cycle: {}", cycle);

    Ok(())
}

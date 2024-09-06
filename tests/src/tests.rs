use ckb_cinnabar::calculator::{
    instruction::Instruction,
    operation::{
        basic::AddOutputCell,
        component::hardcoded::Name,
        spore::{make_cluster_data, make_spore_data},
    },
    re_exports::{
        ckb_sdk::{Address, NetworkType},
        ckb_types::{
            core::HeaderView,
            packed::{OutPoint, Script},
            prelude::Unpack,
        },
        eyre, tokio,
    },
    simulation::{
        always_success_script, fake_header_view, fake_outpoint, random_hash,
        AddFakeAlwaysSuccessCelldep, AddFakeContractCelldep, AddFakeContractCelldepByName,
        AddFakeInputCell, FakeRpcClient, TransactionSimulator, ALWAYS_SUCCESS, DEFUALT_MAX_CYCLES,
    },
    skeleton::{CellOutputEx, ScriptEx, TransactionSkeleton},
    TransactionCalculator,
};
use dao_certificate_calculator::{
    calculator::{
        dao_deposit_with_certificate, dao_withdraw_with_certificate, spore_mint_with_certificate,
    },
    config::{DAO_CERTIFICATE_CHECK_NAME, DAO_CERTIFICATE_NAME},
};

mod dao_deposit {
    use super::*;

    #[tokio::test]
    async fn test_dao_deposit_with_certificate() -> eyre::Result<()> {
        let depositer = always_success_script(vec![0]);

        // prepare celldeps
        let prepare_celldep = Instruction::<FakeRpcClient>::new(vec![
            Box::new(AddFakeAlwaysSuccessCelldep {}),
            Box::new(AddFakeContractCelldep {
                name: "dao".to_string(),
                contract_data: ALWAYS_SUCCESS.to_vec(),
                type_id_args: None,
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: DAO_CERTIFICATE_NAME.to_string(),
                type_id_args: Some([0u8; 32].into()),
                contract_binary_path: "../build/release".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: Name::TypeBurn.to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
        ]);

        // prepare input
        let prepare_input = Instruction::<FakeRpcClient>::new(vec![Box::new(AddFakeInputCell {
            lock_script: depositer.clone().into(),
            type_script: None,
            data: vec![],
            capacity: 0,
            absolute_capacity: false,
        })]);

        // run
        let rpc = FakeRpcClient::default();
        let depositer = Address::new(NetworkType::Dev, depositer.into(), true);
        let dao_capacity = 1000u64;
        let deposit = dao_deposit_with_certificate::<FakeRpcClient>(depositer, dao_capacity);
        let cycle = TransactionSimulator::default()
            .async_verify(
                &rpc,
                vec![prepare_celldep, prepare_input, deposit],
                DEFUALT_MAX_CYCLES,
            )
            .await?;
        println!("cycle: {}", cycle);

        Ok(())
    }
}

mod spore_mint {
    use super::*;

    fn prepare_fake_lock_proxy_input_cell(
        owner: &Script,
        rpc: &mut FakeRpcClient,
        skeleton: &TransactionSkeleton,
    ) -> eyre::Result<()> {
        let dao_certificate_check_script =
            ScriptEx::from((DAO_CERTIFICATE_CHECK_NAME.to_owned(), vec![]));
        let lock_proxy_script = ScriptEx::from((
            Name::LockProxy.to_string(),
            owner.calc_script_hash().raw_data().to_vec(),
        ));
        let fake_lock_proxy_input_cell = CellOutputEx::new_from_scripts(
            dao_certificate_check_script.clone().to_script(&skeleton)?,
            Some(lock_proxy_script.to_script(&skeleton)?),
            vec![],
            None,
        )?;
        rpc.insert_fake_cell(fake_outpoint(), fake_lock_proxy_input_cell);
        Ok(())
    }

    fn prepare_fake_dao_certificate_input_cell(
        header: &HeaderView,
        owner: &Script,
        rpc: &mut FakeRpcClient,
        skeleton: &TransactionSkeleton,
    ) -> eyre::Result<OutPoint> {
        let dao_certificate_type =
            ScriptEx::from((DAO_CERTIFICATE_NAME.to_string(), vec![0u8; 32]));
        let dao_capacity = 1000u64.to_le_bytes().to_vec();
        let fake_dao_certificate_input_cell = CellOutputEx::new_from_scripts(
            owner.clone().into(),
            Some(dao_certificate_type.to_script(&skeleton)?),
            dao_capacity,
            None,
        )?;
        let outpoint = fake_outpoint();
        rpc.insert_tx_status(
            outpoint.tx_hash().unpack(),
            header.hash().unpack(),
            header.number(),
        )
        .insert_fake_header(header.clone())
        .insert_fake_cell(outpoint.clone(), fake_dao_certificate_input_cell);
        Ok(outpoint)
    }

    fn prepare_fake_cluster_celldep_cell(
        cluster_id: [u8; 32],
        rpc: &mut FakeRpcClient,
        skeleton: &TransactionSkeleton,
    ) -> eyre::Result<()> {
        let dao_certificate_check_script =
            ScriptEx::from((DAO_CERTIFICATE_CHECK_NAME.to_owned(), vec![]));
        let cluster_script = ScriptEx::from(("cluster".to_string(), cluster_id.to_vec()));
        let fake_cluster_celldep_cell = CellOutputEx::new_from_scripts(
            dao_certificate_check_script.to_script(&skeleton)?,
            Some(cluster_script.to_script(&skeleton)?),
            make_cluster_data("JoyDAO", b"dao-certificate"),
            None,
        )?;
        rpc.insert_fake_cell(fake_outpoint(), fake_cluster_celldep_cell);
        Ok(())
    }

    async fn generate_celldep_prepared_skeleton(
        rpc: &mut FakeRpcClient,
    ) -> eyre::Result<TransactionSkeleton> {
        let prepare_celldep = Instruction::<FakeRpcClient>::new(vec![
            Box::new(AddFakeAlwaysSuccessCelldep {}),
            Box::new(AddFakeContractCelldepByName {
                contract: DAO_CERTIFICATE_NAME.to_string(),
                type_id_args: Some([0u8; 32].into()),
                contract_binary_path: "../build/release".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: DAO_CERTIFICATE_CHECK_NAME.to_string(),
                type_id_args: Some([1u8; 32].into()),
                contract_binary_path: "../build/release".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: "cluster".to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: "spore".to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: Name::LockProxy.to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: Name::TypeBurn.to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
        ]);
        let (skeleton, _) = TransactionCalculator::default()
            .instruction(prepare_celldep)
            .new_skeleton(&rpc)
            .await?;
        Ok(skeleton)
    }

    #[tokio::test]
    async fn test_spore_mint_with_certificate() -> eyre::Result<()> {
        let mut rpc = FakeRpcClient::default();
        let skeleton = generate_celldep_prepared_skeleton(&mut rpc).await?;
        let depositer = always_success_script(vec![0]);
        let cluster_id = random_hash();
        let header = fake_header_view(10086, 1000, 20);

        // prepare fake lock-proxy input cell
        prepare_fake_lock_proxy_input_cell(&depositer, &mut rpc, &skeleton)?;

        // prepare dao-certificate input cell
        let outpoint =
            prepare_fake_dao_certificate_input_cell(&header, &depositer, &mut rpc, &skeleton)?;

        // prepare cluster celldep cell
        prepare_fake_cluster_celldep_cell(cluster_id, &mut rpc, &skeleton)?;

        // run
        let depositer = Address::new(NetworkType::Dev, depositer.into(), true);
        let mint = spore_mint_with_certificate::<FakeRpcClient>(depositer, cluster_id.into());
        let cycle = TransactionSimulator::default()
            .skeleton(skeleton)
            .link_cell_to_header(outpoint, header)
            .async_verify(&rpc, vec![mint], DEFUALT_MAX_CYCLES)
            .await?;
        println!("cycle: {}", cycle);

        Ok(())
    }
}

mod dao_withdraw {
    use super::*;

    fn prepare_type_burn_chained_input_cells(
        owner: Script,
        spore_id: [u8; 32],
        rpc: &mut FakeRpcClient,
        skeleton: &TransactionSkeleton,
    ) -> eyre::Result<()> {
        // spore cell
        let spore_type =
            ScriptEx::from(("spore".to_string(), spore_id.to_vec())).to_script(skeleton)?;
        let spore_type_hash = spore_type.calc_script_hash().raw_data();
        let spore_data = make_spore_data("dob/1", b"hello, spore", None);
        let fake_spore_cell = CellOutputEx::new_from_scripts(
            owner.clone().into(),
            Some(spore_type),
            spore_data,
            None,
        )?;
        rpc.insert_fake_cell(fake_outpoint(), fake_spore_cell);

        // dao-certificate cell that linked to spore cell
        let dao_certificate_type_burn_lock =
            ScriptEx::from((Name::TypeBurn.to_string(), spore_type_hash.to_vec()));
        let dao_certificate_type =
            ScriptEx::from((DAO_CERTIFICATE_NAME.to_string(), vec![0u8; 32]))
                .to_script(skeleton)?;
        let dao_certificate_type_hash = dao_certificate_type.calc_script_hash().raw_data();
        let dao_capacity = 1000u64.to_le_bytes().to_vec();
        let fake_dao_certificate_cell = CellOutputEx::new_from_scripts(
            dao_certificate_type_burn_lock.to_script(skeleton)?,
            Some(dao_certificate_type),
            dao_capacity,
            None,
        )?;
        rpc.insert_fake_cell(fake_outpoint(), fake_dao_certificate_cell);

        // dao cell that linked to dao-certificate cell
        let dao_type_burn_lock = ScriptEx::from((
            Name::TypeBurn.to_string(),
            dao_certificate_type_hash.to_vec(),
        ));
        let dao_type = ScriptEx::from(("dao".to_string(), vec![]));
        let block_number = 10086u64.to_le_bytes().to_vec();
        let fake_dao_cell = CellOutputEx::new_from_scripts(
            dao_type_burn_lock.to_script(skeleton)?,
            Some(dao_type.to_script(skeleton)?),
            block_number,
            None,
        )?;
        rpc.insert_fake_cell(fake_outpoint(), fake_dao_cell);

        Ok(())
    }

    async fn generate_celldep_prepared_skeleton(
        rpc: &mut FakeRpcClient,
    ) -> eyre::Result<TransactionSkeleton> {
        let prepare_celldep = Instruction::<FakeRpcClient>::new(vec![
            Box::new(AddFakeAlwaysSuccessCelldep {}),
            Box::new(AddFakeContractCelldep {
                name: "dao".to_string(),
                contract_data: ALWAYS_SUCCESS.to_vec(),
                type_id_args: None,
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: DAO_CERTIFICATE_NAME.to_string(),
                type_id_args: Some([0u8; 32].into()),
                contract_binary_path: "../build/release".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: Name::TypeBurn.to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
            Box::new(AddFakeContractCelldepByName {
                contract: "spore".to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
            // note: used to calculate native cluster code hash
            Box::new(AddFakeContractCelldepByName {
                contract: "cluster".to_string(),
                type_id_args: None,
                contract_binary_path: "./binaries".to_string(),
            }),
        ]);

        let (skeleton, _) = TransactionCalculator::default()
            .instruction(prepare_celldep)
            .new_skeleton(&rpc)
            .await?;
        Ok(skeleton)
    }

    #[tokio::test]
    async fn test_dao_withdraw_with_certificate() -> eyre::Result<()> {
        let mut rpc = FakeRpcClient::default();
        let skeleton = generate_celldep_prepared_skeleton(&mut rpc).await?;
        let depositer = always_success_script(vec![0]);
        let spore_id = random_hash();

        // prepare type-burn chained input cells
        prepare_type_burn_chained_input_cells(depositer.clone(), spore_id, &mut rpc, &skeleton)?;

        // prepare fake output cell
        let prepare_output = Instruction::<FakeRpcClient>::new(vec![Box::new(AddOutputCell {
            lock_script: depositer.clone().into(),
            type_script: None,
            data: vec![],
            absolute_capacity: false,
            capacity: 0,
            type_id: false,
        })]);

        // run
        let depositer = Address::new(NetworkType::Dev, depositer.into(), true);
        let withdraw = dao_withdraw_with_certificate::<FakeRpcClient>(depositer, spore_id.into());
        let cycle = TransactionSimulator::default()
            .skeleton(skeleton)
            .async_verify(&rpc, vec![withdraw, prepare_output], DEFUALT_MAX_CYCLES)
            .await?;
        println!("cycle: {}", cycle);

        Ok(())
    }
}

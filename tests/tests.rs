use {
    serial_test::serial,
    skeleton::{id, instruction},
    solana_cli_config::Config as SolanaConfig,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        bpf_loader_upgradeable,
        epoch_schedule::{EpochSchedule, MINIMUM_SLOTS_PER_EPOCH},
        pubkey::Pubkey,
        signature::{write_keypair_file, Keypair, Signer},
        system_instruction, system_program,
        transaction::Transaction,
        vote::{
            self,
            instruction::CreateVoteAccountConfig,
            state::{VoteInit, VoteState, VoteStateVersions},
        },
    },
    solana_test_validator::{TestValidator, TestValidatorGenesis, UpgradeableProgramInfo},
    spl_token_client::client::{ProgramClient, ProgramRpcClient, ProgramRpcClientSendTransaction},
    std::{path::PathBuf, sync::Arc},
    tempfile::NamedTempFile,
};

// this test setup will create a validator with a payer and a vote account
// this is done so we dont have to rely on the user running it, having their config in the default location, etc
// the one thing this doesnt do is you need to run `cargo build-sbf` yourself to build the bpf program

type PClient = Arc<dyn ProgramClient<ProgramRpcClientSendTransaction>>;

#[allow(dead_code)]
pub struct Env {
    pub rpc_client: Arc<RpcClient>,
    pub program_client: PClient,
    pub payer: Keypair,
    pub keypair_file_path: String,
    pub config_file_path: String,
    pub vote_account: Pubkey,

    // persist in struct so they dont scope out but callers dont need to make them
    validator: TestValidator,
    keypair_file: NamedTempFile,
    config_file: NamedTempFile,
}

async fn setup() -> Env {
    // start test validator
    let (validator, payer) = start_validator().await;

    // make clients
    let rpc_client = Arc::new(validator.get_async_rpc_client());
    let program_client: PClient = Arc::new(ProgramRpcClient::new(
        rpc_client.clone(),
        ProgramRpcClientSendTransaction,
    ));

    // write the payer to disk
    let keypair_file = NamedTempFile::new().unwrap();
    write_keypair_file(&payer, &keypair_file).unwrap();

    // write a full config file with our rpc and payer to disk
    let config_file = NamedTempFile::new().unwrap();
    let config_file_path = config_file.path().to_str().unwrap();
    let solana_config = SolanaConfig {
        json_rpc_url: validator.rpc_url(),
        websocket_url: validator.rpc_pubsub_url(),
        keypair_path: keypair_file.path().to_str().unwrap().to_string(),
        ..SolanaConfig::default()
    };
    solana_config.save(config_file_path).unwrap();

    // make vote account
    let vote_account = create_vote_account(&program_client, &payer, &payer.pubkey()).await;

    Env {
        rpc_client,
        program_client,
        payer,
        keypair_file_path: keypair_file.path().to_str().unwrap().to_string(),
        config_file_path: config_file_path.to_string(),
        vote_account,
        validator,
        keypair_file,
        config_file,
    }
}

async fn start_validator() -> (TestValidator, Keypair) {
    solana_logger::setup_with_default(
        "solana_rbpf::vm=debug,\
         solana_runtime::message_processor=debug,\
         solana_runtime::system_instruction_processor=trace,\
         solana_program_test=info",
    );

    let mut test_validator_genesis = TestValidatorGenesis::default();

    test_validator_genesis.epoch_schedule(EpochSchedule::custom(
        MINIMUM_SLOTS_PER_EPOCH,
        MINIMUM_SLOTS_PER_EPOCH,
        false,
    ));

    test_validator_genesis.add_upgradeable_programs_with_path(&[UpgradeableProgramInfo {
        program_id: id(),
        loader: bpf_loader_upgradeable::id(),
        program_path: PathBuf::from("target/deploy/skeleton.so"),
        upgrade_authority: Pubkey::default(),
    }]);

    test_validator_genesis.start_async().await
}

async fn create_vote_account(
    program_client: &PClient,
    payer: &Keypair,
    withdrawer: &Pubkey,
) -> Pubkey {
    let validator = Keypair::new();
    let vote_account = Keypair::new();
    let voter = Keypair::new();

    let zero_rent = program_client
        .get_minimum_balance_for_rent_exemption(0)
        .await
        .unwrap();

    let vote_rent = program_client
        .get_minimum_balance_for_rent_exemption(VoteState::size_of() * 2)
        .await
        .unwrap();

    let blockhash = program_client.get_latest_blockhash().await.unwrap();

    let mut instructions = vec![system_instruction::create_account(
        &payer.pubkey(),
        &validator.pubkey(),
        zero_rent,
        0,
        &system_program::id(),
    )];
    instructions.append(&mut vote::instruction::create_account_with_config(
        &payer.pubkey(),
        &vote_account.pubkey(),
        &VoteInit {
            node_pubkey: validator.pubkey(),
            authorized_voter: voter.pubkey(),
            authorized_withdrawer: *withdrawer,
            ..VoteInit::default()
        },
        vote_rent,
        CreateVoteAccountConfig {
            space: VoteStateVersions::vote_state_size_of(true) as u64,
            ..Default::default()
        },
    ));

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));

    transaction
        .try_partial_sign(&vec![payer], blockhash)
        .unwrap();
    transaction
        .try_partial_sign(&vec![&validator, &vote_account], blockhash)
        .unwrap();

    program_client.send_transaction(&transaction).await.unwrap();

    vote_account.pubkey()
}

#[tokio::test]
#[serial]
async fn test_something() {
    let env = setup().await;

    let account = Pubkey::new_unique();
    let instruction = instruction::do_something(&id(), &account);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&env.payer.pubkey()));

    let blockhash = env.program_client.get_latest_blockhash().await.unwrap();
    transaction
        .try_partial_sign(&vec![&env.payer], blockhash)
        .unwrap();

    env.program_client
        .send_transaction(&transaction)
        .await
        .unwrap();
}

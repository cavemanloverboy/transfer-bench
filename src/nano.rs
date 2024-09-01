use std::{env, path::Path};

use litesvm::LiteSVM;
use nanotoken::{
    consts::CONFIG_ACCOUNT,
    ix::{InitializeAccountArgs, InitializeMintArgs, MintArgs, Tag},
    Mint, ProgramConfig, TokenAccount,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    system_program, system_transaction,
    transaction::Transaction,
};

pub fn initialize_program_and_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    // Initialize config
    let config_keypair =
        read_keypair_file(Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("config.json"))
            .unwrap();
    let config = config_keypair.pubkey();
    let create_config = system_transaction::create_account(
        &payer,
        &config_keypair,
        svm.latest_blockhash(),
        Rent::default().minimum_balance(ProgramConfig::space()),
        ProgramConfig::space() as u64,
        &nanotoken::ID,
    );
    svm.send_transaction(create_config).unwrap();

    // Initialize mint
    let mint_keypair = Keypair::new();
    let mint = mint_keypair.pubkey();
    let create_mint = system_transaction::create_account(
        &payer,
        &mint_keypair,
        svm.latest_blockhash(),
        Rent::default().minimum_balance(Mint::space()),
        Mint::space() as u64,
        &nanotoken::ID,
    );
    svm.send_transaction(create_mint).unwrap();

    // Initialize config
    let ix_data = (Tag::InitializeConfig as u64).to_le_bytes().to_vec();

    let accounts = vec![
        AccountMeta::new(config, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(payer.pubkey(), false),
    ];
    let instruction = Instruction {
        program_id: nanotoken::ID,
        accounts,
        data: ix_data,
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(transaction).unwrap();

    // Initialize mint
    let mut ix_data = vec![0; 8 + InitializeMintArgs::size()];
    ix_data[0..8].copy_from_slice(&(Tag::InitializeMint as u64).to_le_bytes());
    let InitializeMintArgs {
        authority,
        decimals,
    } = bytemuck::try_from_bytes_mut(&mut ix_data[8..]).unwrap();
    *authority = payer.pubkey();
    *decimals = 6;

    let accounts = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new(config, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new(payer.pubkey(), false),
    ];
    let instruction = Instruction {
        program_id: nanotoken::ID,
        accounts,
        data: ix_data,
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(transaction).unwrap();

    mint
}

pub fn initialize_token_account(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    payer: &Keypair,
    owner_key: &Pubkey,
    mint_amount: u64,
) -> Pubkey {
    // Initialize token account AND mint
    let mut ix_data = vec![0; 8 + InitializeAccountArgs::size() + 8 + MintArgs::size()];
    let (token_account, token_account_bump) = TokenAccount::address(0, &payer.pubkey());
    {
        ix_data[0..8].copy_from_slice(&(Tag::InitializeAccount as u64).to_le_bytes());
        let InitializeAccountArgs { owner, mint, bump } =
            bytemuck::try_from_bytes_mut(&mut ix_data[8..8 + InitializeAccountArgs::size()])
                .unwrap();
        *owner = *owner_key;
        *mint = 0;
        *bump = token_account_bump as u64;
        ix_data[8 + InitializeAccountArgs::size()..8 + InitializeAccountArgs::size() + 8]
            .copy_from_slice(&(Tag::Mint as u64).to_le_bytes());
        let MintArgs { amount } =
            bytemuck::try_from_bytes_mut(&mut ix_data[8 + InitializeAccountArgs::size() + 8..])
                .unwrap();
        *amount = mint_amount;
    }
    let accounts = vec![
        // create
        AccountMeta::new(token_account, false),
        // mint
        AccountMeta::new(token_account, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new(payer.pubkey(), true),
        // remainder
        AccountMeta::new(CONFIG_ACCOUNT, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new(payer.pubkey(), true),
    ];
    let instruction = Instruction {
        program_id: nanotoken::ID,
        accounts,
        data: ix_data,
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(transaction).unwrap();

    token_account
}

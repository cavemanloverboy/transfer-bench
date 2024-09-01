use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction, transaction::Transaction,
};
use spl_token::{
    solana_program::program_pack::Pack,
    state::{Account, Mint},
};

pub fn initialize_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    // Create account + initialize mint instruction
    let mint = Keypair::new();
    let create_account_instruction = system_instruction::create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        LAMPORTS_PER_SOL,
        Mint::LEN as u64,
        &spl_token::ID,
    );
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::ID,
        &mint.pubkey(),
        &payer.pubkey(),
        None,
        6,
    )
    .unwrap();

    // Sign and execute transaction
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_mint_ix],
        Some(&payer.pubkey()),
        &[payer, &mint],
        svm.latest_blockhash(),
    );

    svm.send_transaction(transaction).unwrap();

    mint.pubkey()
}

pub fn initialize_token_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Pubkey {
    // Create account + initialize account + mint instruction
    let token_account_keypair = Keypair::new();
    let create_account_instruction = system_instruction::create_account(
        &payer.pubkey(),
        &token_account_keypair.pubkey(),
        LAMPORTS_PER_SOL,
        Account::LEN as u64,
        &spl_token::ID,
    );
    let initialize_account_ix = spl_token::instruction::initialize_account(
        &spl_token::ID,
        &token_account_keypair.pubkey(),
        mint,
        owner,
    )
    .unwrap();
    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        mint,
        &token_account_keypair.pubkey(),
        &payer.pubkey(),
        &[&payer.pubkey()],
        amount,
    )
    .unwrap();

    // Sign and execute transaction
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction, initialize_account_ix, mint_ix],
        Some(&payer.pubkey()),
        &[payer, &token_account_keypair],
        svm.latest_blockhash(),
    );

    svm.send_transaction(transaction).unwrap();

    token_account_keypair.pubkey()
}

use litesvm::LiteSVM;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer};

pub mod nano;
pub mod spl;

pub fn new_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new()
        .with_sigverify(false)
        .with_builtins()
        .with_spl_programs()
        .with_transaction_history(0);
    svm.add_program_from_file(nanotoken::ID, "nanotoken.so")
        .unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
        .unwrap();

    (svm, payer)
}

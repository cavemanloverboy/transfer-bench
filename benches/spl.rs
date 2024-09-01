use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer, transaction::Transaction,
};

fn nano(c: &mut Criterion) {
    // Initialize svm
    let (mut svm, payer) = transfer_bench::new_svm();

    // Initialize program and mint
    let mint = transfer_bench::spl::initialize_mint(&mut svm, &payer);

    // Initialize token accounts
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), LAMPORTS_PER_SOL).unwrap();
    let token_account_1 = transfer_bench::spl::initialize_token_account(
        &mut svm,
        &payer,
        &payer.pubkey(),
        &mint,
        1_000_000_000,
    );
    let token_account_2 =
        transfer_bench::spl::initialize_token_account(&mut svm, &payer, &owner.pubkey(), &mint, 0);

    let mut g = c.benchmark_group("transfer");
    g.throughput(Throughput::Elements(1));

    // Transfer
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &token_account_1,
        &token_account_2,
        &payer.pubkey(),
        &[&payer.pubkey()],
        1,
    )
    .unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );

    g.bench_function("spl", |b| {
        b.iter(|| {
            svm.send_transaction(transaction.clone()).unwrap();
        })
    });
}

criterion_group!(transfer, nano);
criterion_main!(transfer);

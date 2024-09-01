use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use nanotoken::ix::{Tag, TransferArgs};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

fn nano(c: &mut Criterion) {
    // Initialize svm
    let (mut svm, payer) = transfer_bench::new_svm();

    // Initialize program and mint
    let mint = transfer_bench::nano::initialize_program_and_mint(&mut svm, &payer);

    // Initialize token accounts
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), LAMPORTS_PER_SOL).unwrap();
    let token_account_1 = transfer_bench::nano::initialize_token_account(
        &mut svm,
        &mint,
        &payer,
        &payer.pubkey(),
        1_000_000_000,
    );
    let token_account_2 =
        transfer_bench::nano::initialize_token_account(&mut svm, &mint, &owner, &owner.pubkey(), 0);

    let mut g = c.benchmark_group("transfer");
    g.throughput(Throughput::Elements(1));

    // Transfer instruction
    // transfer fast path
    let mut ix_data = vec![0; 8 + TransferArgs::size()];
    {
        ix_data[0..8].copy_from_slice(&(Tag::Transfer as u64).to_le_bytes());
        let TransferArgs { amount } =
            bytemuck::try_from_bytes_mut(&mut ix_data[8..8 + TransferArgs::size()]).unwrap();
        *amount = 5;
    }
    let accounts = vec![
        // transfer
        AccountMeta::new(token_account_1, false),
        AccountMeta::new(token_account_2, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
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

    g.bench_function("nanotoken", |b| {
        b.iter(|| {
            svm.send_transaction(transaction.clone()).unwrap();
        })
    });
}

criterion_group!(transfer, nano);
criterion_main!(transfer);

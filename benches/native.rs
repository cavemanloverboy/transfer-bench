use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use solana_sdk::{pubkey::Pubkey, system_transaction};

fn nano(c: &mut Criterion) {
    // Initialize svm
    let (mut svm, payer) = transfer_bench::new_svm();

    let mut g = c.benchmark_group("transfer");
    g.throughput(Throughput::Elements(1));

    // Destination
    let dest = Pubkey::new_unique();

    // Transfer
    let transaction = system_transaction::transfer(&payer, &dest, 1, svm.latest_blockhash());

    g.bench_function("native", |b| {
        b.iter(|| {
            svm.send_transaction(transaction.clone()).unwrap();
        })
    });
}

criterion_group!(transfer, nano);
criterion_main!(transfer);

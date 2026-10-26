use std::{collections::HashMap, future::Future, sync::Arc};

use sea_orm::DatabaseTransaction;

tokio::task_local! {
    static TRANSACTIONS: HashMap<usize, Arc<DatabaseTransaction>>;
}

pub(crate) fn current(database: usize) -> Option<Arc<DatabaseTransaction>> {
    TRANSACTIONS.try_with(|transactions| transactions.get(&database).cloned()).ok().flatten()
}

pub(crate) async fn scope<T>(
    database: usize,
    transaction: Arc<DatabaseTransaction>,
    future: impl Future<Output = T>,
) -> T {
    let mut transactions = TRANSACTIONS.try_with(Clone::clone).unwrap_or_default();
    transactions.insert(database, transaction);
    TRANSACTIONS.scope(transactions, future).await
}

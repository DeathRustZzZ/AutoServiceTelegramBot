mod common;

use common::fixtures;
use garage_app::{
    CarRepository, ClientRepository, PaymentRepository, PaymentUnitOfWork, RepairRepository,
};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::payment::PgPaymentRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use garage_infra::unit_of_work::payment::PgPaymentUnitOfWork;

#[tokio::test]
async fn payment_unit_of_work_commits_repair_and_payment() {
    let db = common::setup_test_db().await;
    let pool = db.pool();
    let clients = PgClientRepository::new(pool.clone());
    let cars = PgCarRepository::new(pool.clone());
    let repairs = PgRepairRepository::new(pool.clone());
    let payments = PgPaymentRepository::new(pool.clone());
    let client = fixtures::client(1);
    let car = fixtures::car(2, client.id());
    let mut repair = fixtures::repair(3, client.id(), car.id());
    let original_paid_amount = repair.paid_amount();
    let payment = fixtures::payment(4, repair.id(), 5_000);

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");

    repair
        .record_payment(payment.amount(), fixtures::fixed_time(20))
        .expect("payment should be recorded on repair");

    let uow = PgPaymentUnitOfWork::begin(&pool)
        .await
        .expect("payment unit of work should begin");
    uow.repairs()
        .save(&repair)
        .await
        .expect("repair should be saved in transaction");
    uow.payments()
        .save(&payment)
        .await
        .expect("payment should be saved in transaction");
    uow.commit()
        .await
        .expect("payment unit of work should commit");

    let loaded_repair = repairs
        .get(repair.id())
        .await
        .expect("repair should be loaded")
        .expect("repair should exist");
    assert_eq!(loaded_repair.paid_amount(), payment.amount());
    assert_ne!(loaded_repair.paid_amount(), original_paid_amount);

    let loaded_payment = payments
        .get(payment.id())
        .await
        .expect("payment should be loaded")
        .expect("payment should exist after commit");
    assert_eq!(loaded_payment, payment);
}

#[tokio::test]
async fn payment_unit_of_work_rolls_back_repair_and_payment() {
    let db = common::setup_test_db().await;
    let pool = db.pool();
    let clients = PgClientRepository::new(pool.clone());
    let cars = PgCarRepository::new(pool.clone());
    let repairs = PgRepairRepository::new(pool.clone());
    let payments = PgPaymentRepository::new(pool.clone());
    let client = fixtures::client(1);
    let car = fixtures::car(2, client.id());
    let mut repair = fixtures::repair(3, client.id(), car.id());
    let original_paid_amount = repair.paid_amount();
    let payment = fixtures::payment(4, repair.id(), 5_000);

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");

    repair
        .record_payment(payment.amount(), fixtures::fixed_time(20))
        .expect("payment should be recorded on repair");

    let uow = PgPaymentUnitOfWork::begin(&pool)
        .await
        .expect("payment unit of work should begin");
    uow.repairs()
        .save(&repair)
        .await
        .expect("repair should be saved in transaction");
    uow.payments()
        .save(&payment)
        .await
        .expect("payment should be saved in transaction");
    uow.rollback()
        .await
        .expect("payment unit of work should roll back");

    let loaded_repair = repairs
        .get(repair.id())
        .await
        .expect("repair should be loaded")
        .expect("repair should exist");
    assert_eq!(loaded_repair.paid_amount(), original_paid_amount);

    let loaded_payment = payments
        .get(payment.id())
        .await
        .expect("payment lookup should succeed");
    assert!(loaded_payment.is_none());
}

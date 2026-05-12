use garage_app::AppResult;
use garage_domain::{Payment, PaymentComment, PaymentId, RepairId};

use crate::mappers::{map_currency, map_money, map_payment_method};
use crate::models::PaymentRow;

pub fn to_domain(row: &PaymentRow) -> AppResult<Payment> {
    let currency = map_currency("payment", &row.currency)?;

    Payment::restore(
        PaymentId::from_uuid(row.id),
        RepairId::from_uuid(row.repair_id),
        map_money("payment", "amount", row.amount, currency)?,
        map_payment_method(&row.method)?,
        row.comment
            .as_deref()
            .map(PaymentComment::parse)
            .transpose()?
            .flatten(),
        row.paid_at,
        row.created_at,
    )
    .map_err(Into::into)
}

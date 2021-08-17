

use nom::{IResult, branch::alt, bytes::complete::tag, combinator::{map_res}};

use crate::*;

fn parse_client_id(input: &str) -> IResult<&str, ClientID> {
    match nom::sequence::tuple((
        nom::character::complete::char(','),
        nom::character::complete::space1,
        map_res(nom::character::complete::digit1, |client_id| u16::from_str_radix(client_id, 10))
    ))(input)
    {
        Ok((input, (
            _,
            _,
            client_id)
        )) => {
            Ok((input, client_id))
        },
        Err(e) => Err(e),
    }
}

fn parse_tx_id(input: &str) -> IResult<&str, TxID> {
    match nom::sequence::tuple((
        nom::character::complete::char(','),
        nom::character::complete::space1,
        map_res(nom::character::complete::digit1, |tx_id| u32::from_str_radix(tx_id, 10)),
    ))(input)
    {
        Ok((input, (
            _,
            _,
            tx_id,
        ))) => {
            Ok((input, tx_id))
        },
        Err(e) => Err(e),
    }
}

fn parse_tx_amount(input: &str) -> IResult<&str, TxAmount> {
    match nom::sequence::tuple((
        nom::character::complete::char(','),
        nom::character::complete::space1,
        nom::character::complete::digit1,
        nom::character::complete::char('.'),
        nom::character::complete::digit1
    ))(input)
    {
        Ok((input, (
            _,
            _,
            tx_amount1,
            _,
            tx_amount2,
        ))) => {
            let tx_amount = TxAmount::from((tx_amount1, tx_amount2));

            Ok((input, tx_amount))
        },
        Err(e) => Err(e),
    }
}

fn parse_transaction_header(input: &str) -> IResult<&str, (ClientID, TxID)> {
    match nom::sequence::tuple((
        parse_client_id,
        parse_tx_id,
    ))(input) {
        Ok((input, (
            client_id,
            tx_id,
        ))) => {
            Ok((input, (client_id, tx_id)))
        }
        Err(e) => Err(e),
    }
}

fn parse_transaction_complete(input: &str) -> IResult<&str, (ClientID, TxID, TxAmount)> {
    match nom::sequence::tuple((
        parse_transaction_header,
        parse_tx_amount,
    ))(input) {
        Ok((input, (
            (client_id, tx_id),
            tx_amount,
        ))) => {
            Ok((input, (client_id, tx_id, tx_amount)))
        }
        Err(e) => Err(e),
    }
}

fn parse_withdrawal(input: &str) -> IResult<&str, LedgerItem> {
    let (input, _) = tag("withdrawal")(input)?;

    let (input, (client_id, tx_id, tx_amount)) = parse_transaction_complete(input)?;

    Ok((input, LedgerItem { client_id, tx_id, action: LedgerAction::Withdrawal(tx_amount) }))
}

fn parse_deposit(input: &str) -> IResult<&str, LedgerItem> {
    let (input, _) = tag("deposit")(input)?;

    let (input, (client_id, tx_id, tx_amount)) = parse_transaction_complete(input)?;

    Ok((input, LedgerItem { client_id, tx_id, action: LedgerAction::Deposit(tx_amount) }))
}

fn parse_dispute(input: &str) -> IResult<&str, LedgerItem> {
    let (input, _) = tag("dispute")(input)?;

    let (input, (client_id, tx_id)) = parse_transaction_header(input)?;

    Ok((input, LedgerItem { client_id, tx_id, action: LedgerAction::Dispute }))
}

fn parse_resolve(input: &str) -> IResult<&str, LedgerItem> {
    let (input, _) = tag("resolve")(input)?;

    let (input, (client_id, tx_id)) = parse_transaction_header(input)?;

    Ok((input, LedgerItem { client_id, tx_id, action: LedgerAction::Resolve }))
}

fn parse_chargeback(input: &str) -> IResult<&str, LedgerItem> {
    let (input, _) = tag("chargeback")(input)?;

    let (input, (client_id, tx_id)) = parse_transaction_header(input)?;

    Ok((input, LedgerItem { client_id, tx_id, action: LedgerAction::Chargeback }))
}

fn parse_internal(input: &str) -> IResult<&str, LedgerItem> {
    alt((parse_withdrawal, parse_deposit, parse_dispute, parse_chargeback, parse_resolve))(input)
}

pub fn parse_line(input: &str) -> Result<LedgerItem, &'static str> {
    if let Ok((_input, item)) = parse_internal(input) {
        Ok(item)
    } else {
        Err("Invalid item")
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use super::parse_line;

    #[test]
    fn deposit() {
        let tx = parse_line("deposit, 1, 1, 1.10").unwrap();

        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.action, LedgerAction::Deposit(TxAmount::new(11000)));
    }

    #[test]
    fn withdrawal() {
        let tx = parse_line("withdrawal, 1, 1, 1.10").unwrap();

        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.action, LedgerAction::Withdrawal(TxAmount::new(11000)));
    }

    #[test]
    fn dispute() {
        let tx = parse_line("dispute, 1, 1").unwrap();

        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.action, LedgerAction::Dispute);
    }

    #[test]
    fn resolve() {
        let tx = parse_line("resolve, 1, 1").unwrap();

        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.action, LedgerAction::Resolve);
    }

    #[test]
    fn chargeback() {
        let tx = parse_line("chargeback, 1, 1").unwrap();

        assert_eq!(tx.client_id, 1);
        assert_eq!(tx.tx_id, 1);
        assert_eq!(tx.action, LedgerAction::Chargeback);
    }
}
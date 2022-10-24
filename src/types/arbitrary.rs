use crate::types::events::{AmendOrderEvent, FillEvent, PlaceOrderEvent};
use crate::types::order::{Id, Instrument, Side, TimeInForce};
use aptos_api_types::{Address, U64};
use aptos_sdk::types::account_address::AccountAddress;
use arbitrary::{Arbitrary, Error as ArbitraryError, Unstructured};

impl<'a> arbitrary::Arbitrary<'a> for Id {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let (creation_size, _) = u64::size_hint(0);
        let mut creation_num = Unstructured::new(u.bytes(creation_size)?);
        let creation_num = <u64 as Arbitrary>::arbitrary(&mut creation_num)?;
        let creation_num = U64(creation_num);

        let addr = u.bytes(u.len())?;
        let addr = AccountAddress::from_bytes(addr).map_err(|_| ArbitraryError::IncorrectFormat)?;
        let addr = Address::from(addr);
        Ok(Self { creation_num, addr })
    }
}

impl<'a> arbitrary::Arbitrary<'a> for Instrument {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let owner = u.bytes(AccountAddress::LENGTH)?;
        let owner =
            AccountAddress::from_bytes(owner).map_err(|_| ArbitraryError::IncorrectFormat)?;

        let (u64_size, _) = u64::size_hint(0);

        let price_decimals = u.bytes(1)?[0];
        let size_decimals = u.bytes(1)?[0];

        let mut min_size_amount = Unstructured::new(u.bytes(u64_size)?);
        let min_size_amount = <u64 as Arbitrary>::arbitrary(&mut min_size_amount)?;

        let base_decimals = u.bytes(1)?[0];
        let quote_decimals = u.bytes(1)?[0];

        Ok(Self {
            owner,
            price_decimals,
            size_decimals,
            min_size_amount,
            base_decimals,
            quote_decimals,
        })
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PlaceOrderEvent {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let (id_size, _) = Id::size_hint(0);
        let book_id = u.bytes(id_size)?;
        let mut book_id = Unstructured::new(book_id);
        let book_id = <Id as Arbitrary>::arbitrary(&mut book_id)?;

        let id = u.bytes(id_size)?;
        let mut id = Unstructured::new(id);
        let id = <Id as Arbitrary>::arbitrary(&mut id)?;

        let (side_size, _) = Side::size_hint(0);
        let side = u.bytes(side_size)?;
        let mut side = Unstructured::new(side);
        let side = <Side as Arbitrary>::arbitrary(&mut side)?;

        let (u64_size, _) = u64::size_hint(0);

        let mut price = Unstructured::new(u.bytes(u64_size)?);
        let price = <u64 as Arbitrary>::arbitrary(&mut price)?;

        let mut size = Unstructured::new(u.bytes(u64_size)?);
        let size = <u64 as Arbitrary>::arbitrary(&mut size)?;

        let (time_in_force_size, _) = TimeInForce::size_hint(0);
        let time_in_force = u.bytes(time_in_force_size)?;
        let mut time_in_force = Unstructured::new(time_in_force);
        let time_in_force = <TimeInForce as Arbitrary>::arbitrary(&mut time_in_force)?;

        let (bool_size, _) = bool::size_hint(0);
        let post_only = u.bytes(bool_size)?;
        let mut post_only = Unstructured::new(post_only);
        let post_only = <bool as Arbitrary>::arbitrary(&mut post_only)?;

        let mut time = Unstructured::new(u.bytes(u64_size)?);
        let time = <u64 as Arbitrary>::arbitrary(&mut time)?;

        Ok(Self {
            book_id,
            order_id: id,
            side,
            price,
            size,
            time_in_force,
            post_only,
            time,
        })
    }
}

impl<'a> arbitrary::Arbitrary<'a> for AmendOrderEvent {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let (id_size, _) = Id::size_hint(0);
        let book_id = u.bytes(id_size)?;
        let mut book_id = Unstructured::new(book_id);
        let book_id = <Id as Arbitrary>::arbitrary(&mut book_id)?;

        let order_id = u.bytes(id_size)?;
        let mut order_id = Unstructured::new(order_id);
        let order_id = <Id as Arbitrary>::arbitrary(&mut order_id)?;

        let (id_size, _) = Id::size_hint(0);
        let amend_id = u.bytes(id_size)?;
        let mut amend_id = Unstructured::new(amend_id);
        let amend_id = <Id as Arbitrary>::arbitrary(&mut amend_id)?;

        let (side_size, _) = Side::size_hint(0);
        let side = u.bytes(side_size)?;
        let mut side = Unstructured::new(side);
        let side = <Side as Arbitrary>::arbitrary(&mut side)?;

        let (u64_size, _) = u64::size_hint(0);

        let mut price = Unstructured::new(u.bytes(u64_size)?);
        let price = <u64 as Arbitrary>::arbitrary(&mut price)?;

        let mut size = Unstructured::new(u.bytes(u64_size)?);
        let size = <u64 as Arbitrary>::arbitrary(&mut size)?;

        let mut time = Unstructured::new(u.bytes(u64_size)?);
        let time = <u64 as Arbitrary>::arbitrary(&mut time)?;

        Ok(Self {
            book_id,
            order_id,
            amend_id,
            side,
            price,
            size,
            time,
        })
    }
}

impl<'a> arbitrary::Arbitrary<'a> for FillEvent {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let (id_size, _) = Id::size_hint(0);
        let book_id = u.bytes(id_size)?;
        let mut book_id = Unstructured::new(book_id);
        let book_id = <Id as Arbitrary>::arbitrary(&mut book_id)?;

        let order_id = u.bytes(id_size)?;
        let mut order_id = Unstructured::new(order_id);
        let order_id = <Id as Arbitrary>::arbitrary(&mut order_id)?;

        let (side_size, _) = Side::size_hint(0);
        let side = u.bytes(side_size)?;
        let mut side = Unstructured::new(side);
        let side = <Side as Arbitrary>::arbitrary(&mut side)?;

        let (u64_size, _) = u64::size_hint(0);

        let mut price = Unstructured::new(u.bytes(u64_size)?);
        let price = <u64 as Arbitrary>::arbitrary(&mut price)?;

        let mut fill_size = Unstructured::new(u.bytes(u64_size)?);
        let fill_size = <u64 as Arbitrary>::arbitrary(&mut fill_size)?;

        let mut fee = Unstructured::new(u.bytes(u64_size)?);
        let fee = <u64 as Arbitrary>::arbitrary(&mut fee)?;

        let mut fee_rate = Unstructured::new(u.bytes(u64_size)?);
        let fee_rate = <u64 as Arbitrary>::arbitrary(&mut fee_rate)?;

        let mut time = Unstructured::new(u.bytes(u64_size)?);
        let time = <u64 as Arbitrary>::arbitrary(&mut time)?;

        let mut remaining_size = Unstructured::new(u.bytes(u64_size)?);
        let remaining_size = <u64 as Arbitrary>::arbitrary(&mut remaining_size)?;

        let (bool_size, _) = bool::size_hint(0);
        let is_maker = u.bytes(bool_size)?;
        let mut is_maker = Unstructured::new(is_maker);
        let is_maker = <bool as Arbitrary>::arbitrary(&mut is_maker)?;

        Ok(Self {
            book_id,
            order_id,
            side,
            price,
            fill_size,
            fee,
            fee_rate,
            time,
            remaining_size,
            is_maker,
        })
    }
}

use crate::{
    common::types::CairoUint256,
    db::postgres::process::ProcessEvent,
    events::{EventHandler, HexFieldElement},
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::{models::EmittedEvent, HttpTransport, JsonRpcClient},
};

use super::transfer_single::Erc1155TransferSingle;

pub struct Erc1155TransferBatch {
    pub sender: FieldElement,
    pub recipient: FieldElement,
    pub transfers: Vec<(CairoUint256, CairoUint256)>,
    pub contract_address: HexFieldElement,
    pub block_number: u64,
}

impl Erc1155TransferBatch {
    pub fn new(
        sender: FieldElement,
        recipient: FieldElement,
        transfers: Vec<(CairoUint256, CairoUint256)>,
        contract_address: FieldElement,
        block_number: u64,
    ) -> Self {
        Erc1155TransferBatch {
            sender,
            recipient,
            transfers,
            contract_address: HexFieldElement(contract_address),
            block_number,
        }
    }
}

#[async_trait]
impl ProcessEvent for Erc1155TransferBatch {
    async fn process(&self, handler: &mut EventHandler<'_, '_>) -> Result<()> {
        for transfer in &self.transfers {
            Erc1155TransferSingle::new(
                self.sender,
                self.recipient,
                transfer.0,
                transfer.1,
                self.contract_address.0,
                self.block_number,
            )
            .process(handler)
            .await?;
        }

        Ok(())
    }
}

impl From<&EmittedEvent> for Erc1155TransferBatch {
    fn from(event: &EmittedEvent) -> Self {
        let contract_address = event.from_address;
        let block_number = event.block_number;
        let event_data = &event.data;

        let sender = event_data[1];
        let recipient = event_data[2];

        // Get the length of the token ids array
        let token_length: u32 = event_data[3].try_into().unwrap();
        let token_length = token_length as usize;

        // This is index difference between token id and corresponding amount in the event data array
        let amount_delta = token_length * 2 + 1;

        // Zip token ids and amounts together
        let transfers: Vec<(CairoUint256, CairoUint256)> = event_data[4..(3 + amount_delta)]
            .chunks(2)
            .map(|chunk| CairoUint256::new(chunk[0], chunk[1]))
            .zip(
                event_data[(4 + amount_delta)..]
                    .chunks(2)
                    .map(|chunk| CairoUint256::new(chunk[0], chunk[1])),
            )
            .collect();

        Erc1155TransferBatch::new(sender, recipient, transfers, contract_address, block_number)
    }
}

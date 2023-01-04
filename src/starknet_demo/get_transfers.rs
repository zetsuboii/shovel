use std::collections::HashSet;

use starknet::{
    core::types::{AbiEntry, BlockId, Event, FieldElement},
    macros::{felt, felt_hex},
    providers::{self, Provider, SequencerGatewayProvider},
};

/// felt!("0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9");
const ERC721_TRANSFER_EVENT: FieldElement = FieldElement::from_mont([
    10370298062762752593,
    7288672513944573579,
    6148261015514870755,
    242125613396778233,
]);

/// felt!("0x182d859c0807ba9db63baf8b9d9fdbfeb885d820be6e206b9dab626d995c433");
const ERC1155_TRANSFER_EVENT: FieldElement = FieldElement::from_mont([
    1986363494579022220,
    17146673375846491535,
    6125027481420860397,
    307829215948623223,
]);

/// felt!("0x2563683c757f3abe19c4b7237e2285d8993417ddffe0b54a19eb212ea574b08");
const ERC1155_TRANSFER_BATCH_EVENT: FieldElement = FieldElement::from_mont([
    14114721770411318090,
    10106114908748783105,
    12894248477188639378,
    518981439849896716,
]);

#[derive(Debug)]
enum TransactionType {
    ERC721,
    ERC1155,
    ERC1155Batch,
    Other,
}

fn get_tx_type(event: &Event) -> TransactionType {
    if event.keys.contains(&ERC721_TRANSFER_EVENT) {
        TransactionType::ERC721
    } else if event.keys.contains(&ERC1155_TRANSFER_EVENT) {
        TransactionType::ERC1155
    } else if event.keys.contains(&ERC1155_TRANSFER_BATCH_EVENT) {
        TransactionType::ERC1155Batch
    } else {
        TransactionType::Other
    }
}

pub async fn run() {
    let provider = SequencerGatewayProvider::starknet_alpha_mainnet();

    // TODO: Replace it with database
    let mut blacklist: HashSet<FieldElement> = HashSet::new();

    let block = provider
        .get_block(BlockId::Hash(felt!(
            "0x02e0a6d54949d54165978adabdc29d2cce780748954668e18784ee34db2c01da"
        )))
        .await
        .unwrap();
    for receipt in block.transaction_receipts.iter() {
        for event in receipt.events.iter() {
            if blacklist.contains(&event.from_address) {
                continue;
            }

            match get_tx_type(&event) {
                TransactionType::ERC721 => {
                    // Get abi
                    let abi = provider
                        .get_code(
                            event.from_address,
                            BlockId::Number(block.block_number.unwrap()),
                        )
                        .await
                        .unwrap()
                        .abi
                        .unwrap();

                    let is_erc721 = abi
                        .iter()
                        // Filter out non-function entries
                        .filter(|abi_entry| match abi_entry {
                            AbiEntry::Function(_) => true,
                            _ => false,
                        })
                        // Get the function name
                        .map(|fn_entry| match fn_entry {
                            AbiEntry::Function(fn_entry) => &fn_entry.name,
                            _ => unreachable!(),
                        })
                        // Check if the function name is "ownerOf"
                        .any(|fn_name| fn_name == "ownerOf" || fn_name == "owner_of");

                    if is_erc721 {
                        dbg!(TransactionType::ERC721, event);
                    } else {
                        blacklist.insert(event.from_address);
                    }
                }
                TransactionType::ERC1155 => {
                    dbg!(TransactionType::ERC1155, event);
                }
                TransactionType::ERC1155Batch => {
                    dbg!(TransactionType::ERC1155Batch, event);
                }
                _ => {}
            }
        }
    }
}
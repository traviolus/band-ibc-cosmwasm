use crate::ContractError;
use base64::decode as b64decode;
use cosmwasm_std::StdResult;
use obi::{OBIDecode, OBIEncode, OBISchema};

#[derive(OBIEncode, OBISchema, Debug)]
pub struct PriceDataInput {
    pub symbol: Vec<String>,
    pub multiplier: u64,
}

impl PriceDataInput {
    pub fn encode_obi(self) -> Result<Vec<u8>, ContractError> {
        let res = OBIEncode::try_to_vec(&self)?;

        Ok(res)
    }
}

#[derive(OBIDecode, OBISchema, Debug)]
pub struct PriceDataOutput {
    pub rates: Vec<u64>,
}

impl PriceDataOutput {
    pub fn decode_obi(encoded: &str) -> StdResult<PriceDataOutput> {
        let res: PriceDataOutput =
            OBIDecode::try_from_slice(b64decode(encoded).unwrap().as_slice()).unwrap();

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_test() {
        let res = PriceDataOutput::decode_obi("AAAAAQAAAAAAHxie").unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn encode_test() {
        let data = PriceDataInput {
            symbol: vec!["LUNA".to_string()],
            multiplier: 1000000,
        };
        let res = data.encode_obi().unwrap();
        println!("{:?}", res);
    }
}

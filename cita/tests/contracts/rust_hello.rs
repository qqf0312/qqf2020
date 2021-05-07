use cita_types::{H256, U256};
use contracts::{native::factory::Contract, tools::method as method_tools};
use evm::action_params::ActionParams;
use evm::storage::Scalar;
use evm::{Error, Ext, GasLeft, ReturnData};

#[derive(Clone)]
pub struct HelloWorld {
    balance: Scalar,
    output: Vec<u8>,
}

impl Contract for HelloWorld {
    fn exec(&mut self, params: &ActionParams, ext: &mut Ext) -> Result<GasLeft, Error> {
        if let Some(ref data) = params.data {
            method_tools::extract_to_u32(&data[..]).and_then(|signature| match signature {
                0 => self.init(params, ext),
                // Register function
                0x832b_4580 => self.balance_get(params, ext),
                0xaa91_543e => self.update(params, ext),
                _ => Err(Error::OutOfGas),
            })
        } else {
            Err(evm::Error::OutOfGas)
        }
    }
    fn create(&self) -> Box<Contract> {
        Box::new(HelloWorld::default())
    }
}

impl Default for HelloWorld {
    fn default() -> Self {
        HelloWorld {
            output: Vec::new(),
            balance: Scalar::new(H256::from(0)),
        }
    }
}

impl HelloWorld {
    fn init(&mut self, _params: &ActionParams, _ext: &mut Ext) -> Result<GasLeft, Error> {
        Ok(GasLeft::Known(U256::from(100)))
    }

    fn balance_get(&mut self, _params: &ActionParams, ext: &mut Ext) -> Result<GasLeft, Error> {
        self.output.resize(32, 0);
        self.balance
            .get(ext)?
            .to_big_endian(self.output.as_mut_slice());

        Ok(GasLeft::NeedsReturn {
            gas_left: U256::from(100),
            data: ReturnData::new(self.output.clone(), 0, self.output.len()),
            apply_state: true,
        })
    }

    fn update(&mut self, params: &ActionParams, ext: &mut Ext) -> Result<GasLeft, Error> {
        self.output.resize(32, 0);

        // Get the params of`update`
        let data = params.data.to_owned().expect("invalid data");
        let amount = U256::from(data.get(4..36).expect("no enough data"));
        let _balance = self.balance.get(ext)?.saturating_add(amount);

        self.balance.set(ext, _balance)?;
        info!("====set balance to {:?}", _balance);

        _balance.to_big_endian(self.output.as_mut_slice());

        Ok(GasLeft::NeedsReturn {
            gas_left: U256::from(100),
            data: ReturnData::new(self.output.clone(), 0, self.output.len()),
            apply_state: true,
        })
    }
}

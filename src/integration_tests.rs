#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{coins, Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use crate::state::{FundDetails, Owner};
    use cw_multi_test::IntoAddr;

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn nft_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(donatio_nfts::execute, donatio_nfts::instantiate, donatio_nfts::query);

        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "uxion";

    fn mock_app() -> App {
        App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_make(USER),
                    coins(600*1000000, NATIVE_DENOM),
                )
                .unwrap();

        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            email: String::from("example@email.com"),
            fullname: String::from("John Doe"),
            title: String::from("Example crowdfund"),
            description: String::from("need some funds"),
            amount_to_be_raised: (5*1000000).to_string(),
            denom: NATIVE_DENOM.into(),
            image_url: String::new(),
            owner: USER.into_addr()
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                ADMIN.into_addr(),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod donations {

        use std::u128;

        use super::*;
        use crate::msg::{ExecuteMsg, QueryMsg};

        #[test]
        fn donate() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg::Donate { message: "Enjoy!".into() };
            let x: u128 = 5*1000000;
            let resp = app.execute_contract(USER.into_addr(), cw_template_contract.addr(), &msg, &[Coin { amount: Uint128::from(x), denom: NATIVE_DENOM.into() }, ]);
            println!("donate response: {:?}", resp);
            assert!(resp.is_ok());
            query_total(&app, cw_template_contract.clone());

            withdraw(app, cw_template_contract);

        }

        fn withdraw(mut app: App, cw_template_contract: CwTemplateContract) {


            let resp = app.execute_contract(USER.into_addr(), cw_template_contract.addr(), &ExecuteMsg::Withdraw {}, &[]);
            println!("withdraw response: {:?}", resp);
            assert!(resp.is_ok());
        }

        #[test]
        fn query_details() {
            let (app, cw_template_contract) = proper_instantiate();
            let details: FundDetails = app.wrap().query_wasm_smart(cw_template_contract.addr(), &QueryMsg::GetDetails {  }).unwrap();

            assert_eq!(details, FundDetails {
                owner: Owner {
                    email: String::from("example@email.com"),
                    fullname: String::from("John Doe"),
                    addr: USER.into_addr()
                },
                title: String::from("Example crowdfund"),
                description: String::from("need some funds"),
                amount_to_be_raised: Uint128::new(5*1000000),
                denom: NATIVE_DENOM.into(),
                image_url: String::new()
            });
        }

        fn query_total(app: &App, cw_template_contract: CwTemplateContract) {
            let details: Uint128 = app.wrap().query_wasm_smart(cw_template_contract.addr(), &QueryMsg::GetTotal {  }).unwrap();

            println!("total: {:?}", details);

        }

    }
}
